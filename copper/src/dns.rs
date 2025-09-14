use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::UdpSocket;

pub type DnsResult<T> = Result<T, &'static str>;

pub const CLOUDFLARE_DNS: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));

pub fn get_host_addr(host: &str, dns: IpAddr) -> DnsResult<IpAddr> {
    let mut question_packet = DnsPacket::question();
    question_packet.push_question(host.into());
    let bytes = question_packet.to_bytes()?;

    let udp_socket = UdpSocket::bind("0.0.0.0:0").map_err(|_| "failed to bind udp socket")?;
    udp_socket
        .connect((dns, 53))
        .map_err(|_| "failed to connect to dns server")?;
    udp_socket
        .send(&bytes)
        .map_err(|_| "failed to send packet")?;

    let mut response_buff = [0u8; 512];
    let response_len = udp_socket
        .recv(&mut response_buff)
        .map_err(|_| "failed to receive packet")?;
    let response = &response_buff[..response_len];
    let response_packet: DnsPacket = response.try_into()?;

    response_packet
        .get_ipaddr(host)
        .ok_or("entry not found in response")
}

#[derive(Clone, Debug)]
pub struct DnsPacket {
    header: Header,
    questions: Vec<Question>,
    answers: Vec<Answer>,
}

impl TryFrom<&[u8]> for DnsPacket {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self::from_bytes(value)?.0)
    }
}

impl DnsPacket {
    pub fn from_bytes(bytes: &[u8]) -> DnsResult<(Self, usize)> {
        let (header, mut index) = Header::from_bytes(bytes, 0)?;
        let mut questions = Vec::new();
        let mut answers = Vec::new();

        for _ in 0..header.qdcount {
            let (question, i) = Question::from_bytes(bytes, index)?;
            index = i;
            questions.push(question);
        }

        for _ in 0..header.ancount {
            let (answer, i) = Answer::from_bytes(bytes, index)?;
            index = i;
            answers.push(answer);
        }

        Ok((
            Self {
                header,
                questions,
                answers,
            },
            index,
        ))
    }

    pub fn question() -> Self {
        Self {
            header: Header::question(),
            questions: Vec::new(),
            answers: Vec::new(),
        }
    }

    pub fn get_ipaddr(&self, host: &str) -> Option<IpAddr> {
        self.get_ipaddr_recursion(host, 16)
    }

    fn get_ipaddr_recursion(&self, host: &str, depth: usize) -> Option<IpAddr> {
        for answer in &self.answers {
            if &answer.name == host {
                return match answer.rdata {
                    RData::IpAddr(addr) => Some(addr),
                    RData::CName(ref name) => {
                        if 0 < depth {
                            self.get_ipaddr_recursion(name, depth - 1)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
            }
        }

        None
    }

    pub fn push_question(&mut self, name: String) {
        let question = Question {
            qname: name,
            qtype: QType::A,
            qclass: QClass::In,
        };
        self.questions.push(question);
        self.header.qdcount += 1;
    }

    pub fn to_bytes(&self) -> DnsResult<Vec<u8>> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.header.to_bytes());

        for question in &self.questions {
            bytes.extend_from_slice(&question.to_bytes()?);
        }

        for answer in &self.answers {
            bytes.extend_from_slice(&answer.to_bytes()?);
        }

        if 512 <= bytes.len() {
            return Err("unsupported packet size");
        }

        Ok(bytes)
    }
}

#[derive(Clone, Copy, Debug)]
struct Header {
    id: [u8; 2],
    qr: bool,
    opcode: Opcode,
    aa: bool,
    tc: bool,
    rd: bool,
    ra: bool,
    rcode: RCode,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

impl Header {
    pub fn rcode(&self) -> RCode {
        self.rcode
    }

    pub fn from_bytes(bytes: &[u8], index: usize) -> DnsResult<(Self, usize)> {
        if bytes.len() < index + 12 {
            return Err("invalid header");
        }
        let bytes = &bytes[index..12];
        let id = bytes[..2].try_into().unwrap();

        let qr = bytes[2] & 0x80 != 0;
        let opcode = ((bytes[2] >> 3) & 0x0f).into();
        let aa = bytes[2] & 0x04 != 0;
        let tc = bytes[2] & 0x02 != 0;
        let rd = bytes[2] & 0x01 != 0;

        let ra = bytes[3] & 0x80 != 0;
        let rcode = (bytes[3] & 0x0f).into();

        let qdcount = u16::from_be_bytes(bytes[4..6].try_into().unwrap());
        let ancount = u16::from_be_bytes(bytes[6..8].try_into().unwrap());
        let nscount = u16::from_be_bytes(bytes[8..10].try_into().unwrap());
        let arcount = u16::from_be_bytes(bytes[10..12].try_into().unwrap());

        Ok((
            Self {
                id,
                qr,
                opcode,
                aa,
                tc,
                rd,
                ra,
                rcode,
                qdcount,
                ancount,
                nscount,
                arcount,
            },
            12,
        ))
    }

    pub fn question() -> Self {
        Self {
            id: [0x46, 0x65], // Fe
            qr: false,
            opcode: Opcode::Query,
            aa: false,
            tc: false,
            rd: true, // Recursion Desired
            ra: false,
            rcode: RCode::Success,
            qdcount: 0,
            ancount: 0,
            nscount: 0,
            arcount: 0,
        }
    }

    pub fn to_bytes(&self) -> [u8; 12] {
        let mut bytes = [0u8; 12];

        bytes[0..2].copy_from_slice(&self.id);

        bytes[2] |= (self.qr as u8) << 7;
        bytes[2] |= (self.opcode.as_u8() & 0x0f) << 3;
        bytes[2] |= (self.aa as u8) << 2;
        bytes[2] |= (self.tc as u8) << 1;
        bytes[2] |= self.rd as u8;

        bytes[3] |= (self.ra as u8) << 7;
        bytes[3] |= self.rcode.as_u8() & 0x0f;

        bytes[4..6].copy_from_slice(&self.qdcount.to_be_bytes());
        bytes[6..8].copy_from_slice(&self.ancount.to_be_bytes());
        bytes[8..10].copy_from_slice(&self.nscount.to_be_bytes());
        bytes[10..12].copy_from_slice(&self.arcount.to_be_bytes());

        bytes
    }
}

#[derive(Clone, Debug)]
pub struct Question {
    qname: String,
    qtype: QType,
    qclass: QClass,
}

impl Question {
    pub fn qname(&self) -> &str {
        &self.qname
    }

    pub fn qtype(&self) -> QType {
        self.qtype
    }

    pub fn qclass(&self) -> QClass {
        self.qclass
    }

    pub fn from_bytes(bytes: &[u8], index: usize) -> DnsResult<(Self, usize)> {
        let (qname, mut index) = get_label(bytes, index)?;
        if bytes.len() < index + 4 {
            return Err("invalid question");
        }
        let qtype = u16::from_be_bytes(bytes[index..index + 2].try_into().unwrap()).into();
        index += 2;
        let qclass = u16::from_be_bytes(bytes[index..index + 2].try_into().unwrap()).into();
        index += 2;
        Ok((
            Self {
                qname,
                qtype,
                qclass,
            },
            index,
        ))
    }

    pub fn to_bytes(&self) -> DnsResult<Vec<u8>> {
        let mut bytes = as_label(&self.qname)?;

        bytes.extend_from_slice(&self.qtype.as_u16().to_be_bytes());
        bytes.extend_from_slice(&self.qclass.as_u16().to_be_bytes());

        Ok(bytes)
    }
}

pub type Answer = ResourceRecord;

#[derive(Clone, Debug)]
pub struct ResourceRecord {
    name: String,
    class: Class,
    ttl: u32,
    rdata: RData,
}

impl ResourceRecord {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn class(&self) -> Class {
        self.class
    }

    pub fn ttl(&self) -> u32 {
        self.ttl
    }

    pub fn rdata(&self) -> &RData {
        &self.rdata
    }

    pub fn from_bytes(bytes: &[u8], index: usize) -> DnsResult<(Self, usize)> {
        let (name, mut index) = get_label(bytes, index)?;
        if bytes.len() <= index + 10 {
            return Err("invalid resource record");
        }
        let rtype = u16::from_be_bytes(bytes[index..index + 2].try_into().unwrap()).into();
        index += 2;
        let class = u16::from_be_bytes(bytes[index..index + 2].try_into().unwrap()).into();
        index += 2;
        let ttl = u32::from_be_bytes(bytes[index..index + 4].try_into().unwrap());
        index += 4;
        let rdlength = u16::from_be_bytes(bytes[index..index + 2].try_into().unwrap());
        index += 2;
        let rdata_raw = bytes
            .get(index..index + rdlength as usize)
            .ok_or("invalid resource record")?;
        let rdata = match rtype {
            RType::A => {
                let ipv4: [u8; 4] = rdata_raw
                    .get(..4)
                    .ok_or("invalid rdata")?
                    .try_into()
                    .unwrap();
                RData::IpAddr(IpAddr::V4(ipv4.into()))
            }
            RType::AAAA => {
                let ipv6: [u8; 16] = rdata_raw
                    .get(..16)
                    .ok_or("invalid rdata")?
                    .try_into()
                    .unwrap();
                RData::IpAddr(IpAddr::V6(ipv6.into()))
            }
            RType::CName => RData::CName(get_label(bytes, index)?.0),
            _ => RData::Unknown,
        };
        index += rdlength as usize;

        Ok((
            Self {
                name,
                class,
                ttl,
                rdata,
            },
            index,
        ))
    }

    pub fn to_bytes(&self) -> DnsResult<Vec<u8>> {
        unimplemented!()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Opcode {
    Query = 0,
    IQuery = 1,
    Status = 2,
    Unknown,
}

impl Opcode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Query,
            1 => Self::IQuery,
            2 => Self::Status,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RCode {
    Success = 0,
    FormatError = 1,
    ServerFailer = 2,
    NoexistedDomain = 3,
    Unimplemented = 4,
    Rejected = 5,
    Unknown,
}

impl RCode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

impl From<u8> for RCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Success,
            1 => Self::FormatError,
            2 => Self::ServerFailer,
            3 => Self::NoexistedDomain,
            4 => Self::Unimplemented,
            5 => Self::Rejected,
            _ => Self::Unknown,
        }
    }
}

pub type QClass = Class;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Class {
    In = 1,
    Ch = 3,
    Hs = 4,
    Unknown,
}

impl Class {
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

impl From<u16> for Class {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::In,
            3 => Self::Ch,
            4 => Self::Hs,
            _ => Self::Unknown,
        }
    }
}

pub type QType = RType;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RType {
    A = 1,
    AAAA = 28,
    CName = 5,
    Unknown,
}

impl RType {
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

impl From<u16> for RType {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::A,
            28 => Self::AAAA,
            5 => Self::CName,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub enum RData {
    IpAddr(IpAddr),
    CName(String),
    Unknown,
}

fn as_label(s: &str) -> DnsResult<Vec<u8>> {
    let mut bytes = Vec::new();

    for part in s.split('.') {
        if 0x7f <= part.len() {
            return Err("too large qname part found");
        }
        if part.len() == 0 {
            break;
        }
        bytes.push(part.len() as u8);

        for byte in part.bytes() {
            bytes.push(byte);
        }
    }

    bytes.push(0);

    Ok(bytes)
}

fn get_label(bytes: &[u8], index: usize) -> DnsResult<(String, usize)> {
    fn append_label(
        bytes: &[u8],
        mut index: usize,
        string: &mut String,
        depth: usize,
    ) -> DnsResult<usize> {
        while let Some(ref_len_u8) = bytes.get(index) {
            let len_u8 = *ref_len_u8;

            if len_u8 == 0 {
                string.pop();
                return Ok(index + 1);
            }

            if len_u8 & 0xc0 == 0xc0 {
                if bytes.len() < index + 2 {
                    return Err("invalid pointer");
                }
                let pointer = [bytes[index] & 0x3f, bytes[index + 1]];
                let pointed_index = u16::from_be_bytes(pointer);
                if 0 < depth {
                    append_label(bytes, pointed_index as usize, string, depth - 1)?;
                    index += 2;
                    return Ok(index);
                } else {
                    return Err("deep recursional pointer");
                }
            } else {
                let len = len_u8 as usize;
                index += 1;
                string.push_str(
                    str::from_utf8(&bytes[index..index + len])
                        .map_err(|_| "invalid utf8 codepoint found")?,
                );
                index += len;
            }

            string.push('.');
        }
        Err("invalid label")
    }

    let mut string = String::new();
    let len = append_label(bytes, index, &mut string, 16)?;
    Ok((string, len))
}
