use std::{collections::BTreeSet, io, net::{Ipv4Addr, SocketAddr, UdpSocket}, time::{Duration, Instant}};

const PORT:u16=27183;

#[cfg(windows)]
fn interfaces()->io::Result<Vec<(Ipv4Addr,u8)>> {
    use windows_sys::Win32::{Foundation::ERROR_BUFFER_OVERFLOW, NetworkManagement::{IpHelper::*, Ndis::IfOperStatusUp}, Networking::WinSock::{AF_INET, SOCKADDR_IN}};
    let mut size=15000u32;
    for _ in 0..3 {
        // u64 storage provides the alignment required by the adapter structures.
        let mut buffer=vec![0u64;(size as usize+7)/8];
        let head=buffer.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();
        let result=unsafe {GetAdaptersAddresses(AF_INET as u32,GAA_FLAG_SKIP_ANYCAST|GAA_FLAG_SKIP_MULTICAST|GAA_FLAG_SKIP_DNS_SERVER,std::ptr::null(),head,&mut size)};
        if result==ERROR_BUFFER_OVERFLOW {continue}
        if result!=0 {return Err(io::Error::from_raw_os_error(result as i32))}
        let mut addresses=Vec::new();
        // All linked addresses belong to buffer, which remains alive during traversal.
        unsafe {
            let mut adapter=head;
            while !adapter.is_null() {
                let a=&*adapter;
                if a.OperStatus==IfOperStatusUp && a.IfType!=24 && a.IfType!=131 {
                    let mut address=a.FirstUnicastAddress;
                    while !address.is_null() {
                        let u=&*address;
                        if !u.Address.lpSockaddr.is_null() && u.Address.iSockaddrLength>=std::mem::size_of::<SOCKADDR_IN>() as i32 {
                            let sa=&*u.Address.lpSockaddr.cast::<SOCKADDR_IN>();
                            if sa.sin_family==AF_INET {
                                let ip=Ipv4Addr::from(sa.sin_addr.S_un.S_addr.to_ne_bytes());
                                if !ip.is_loopback() && !ip.is_unspecified() {addresses.push((ip,u.OnLinkPrefixLength))}
                            }
                        }
                        address=u.Next;
                    }
                }
                adapter=a.Next;
            }
        }
        return Ok(addresses)
    }
    Err(io::Error::new(io::ErrorKind::Other,"Network adapters changed during discovery"))
}

#[cfg(not(windows))]
fn interfaces()->io::Result<Vec<(Ipv4Addr,u8)>> {Ok(Vec::new())}

fn targets(interfaces:&[(Ipv4Addr,u8)])->(Vec<Ipv4Addr>,Vec<Ipv4Addr>) {
    let own:BTreeSet<_>=interfaces.iter().map(|(ip,_)|u32::from(*ip)).collect();
    let mut hosts=BTreeSet::new();let mut broadcasts=BTreeSet::new();
    for &(ip,prefix) in interfaces {
        if prefix==0 || prefix>30 {continue}
        let value=u32::from(ip);let mask=u32::MAX<<(32-prefix);
        let network=value&mask;let broadcast=network|!mask;
        broadcasts.insert(Ipv4Addr::from(broadcast));
        // Bound large networks to the local /24 instead of scanning thousands of hosts.
        let (first,last)=if prefix<24 {(value&0xffffff00,(value&0xffffff00)|255)}else{(network,broadcast)};
        for host in first+1..last {
            if !own.contains(&host) {hosts.insert(Ipv4Addr::from(host));}
        }
    }
    (hosts.into_iter().collect(),broadcasts.into_iter().collect())
}

fn response(packet:&[u8],peer:SocketAddr)->Option<String> {
    if !peer.is_ipv4() || packet.len()!=8 || &packet[..4]!=b"QPO1" || packet[6..8]!=[1,0] {return None}
    let port=u16::from_le_bytes([packet[4],packet[5]]);
    (port>=1024).then(||format!("{}:{port}",peer.ip()))
}

pub fn discover(cancelled:impl Fn()->bool)->io::Result<String> {
    let socket=UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_millis(25)))?;
    let addresses=interfaces().unwrap_or_else(|e|{log::warn!("LAN adapter enumeration failed: {e}");Vec::new()});
    let (hosts,mut broadcasts)=targets(&addresses);
    broadcasts.push(Ipv4Addr::BROADCAST);
    search(&socket,&hosts,&broadcasts,PORT,cancelled)
}

fn search(socket:&UdpSocket,hosts:&[Ipv4Addr],broadcasts:&[Ipv4Addr],port:u16,cancelled:impl Fn()->bool)->io::Result<String> {
    let start=Instant::now();let mut next_broadcast=start;let mut next_batch=start;let mut cursor=0;
    let mut packet=[0u8;64];
    while start.elapsed()<Duration::from_secs(3) {
        if cancelled() {return Err(io::Error::new(io::ErrorKind::Interrupted,"Отключено"))}
        let now=Instant::now();
        if now>=next_broadcast {
            for ip in broadcasts {if let Err(e)=socket.send_to(b"QPD1",(*ip,port)){log::debug!("LAN broadcast {ip}: {e}")}}
            next_broadcast=now+Duration::from_millis(700);
        }
        if now>=next_batch && cursor<hosts.len() {
            let end=(cursor+32).min(hosts.len());
            for ip in &hosts[cursor..end] {if let Err(e)=socket.send_to(b"QPD1",(*ip,port)){log::debug!("LAN probe {ip}: {e}")}}
            cursor=end;next_batch=now+Duration::from_millis(25);
        }
        match socket.recv_from(&mut packet) {
            Ok((n,peer))=>if let Some(endpoint)=response(&packet[..n],peer){return Ok(endpoint)},
            Err(e) if matches!(e.kind(),io::ErrorKind::WouldBlock|io::ErrorKind::TimedOut)=>{},
            Err(e)=>return Err(e),
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound,"Quest не найден в локальной сети"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subnet_targets_are_bounded_and_exclude_own_addresses() {
        let interfaces=[("192.168.1.100".parse().unwrap(),24),("192.168.1.101".parse().unwrap(),24),("10.2.3.4".parse().unwrap(),8)];
        let (hosts,broadcasts)=targets(&interfaces);
        assert_eq!(hosts.len(),505);
        assert!(!hosts.contains(&interfaces[0].0));assert!(!hosts.contains(&interfaces[1].0));
        assert!(hosts.contains(&"192.168.1.102".parse().unwrap()));
        assert!(broadcasts.contains(&"10.255.255.255".parse().unwrap()));
        assert!(!hosts.contains(&"10.2.4.1".parse().unwrap()));
    }
    #[test]
    fn small_subnets_and_invalid_prefixes() {
        let (hosts,_)=targets(&[("192.168.1.1".parse().unwrap(),30)]);
        assert_eq!(hosts,vec!["192.168.1.2".parse::<Ipv4Addr>().unwrap()]);
        assert!(targets(&[(Ipv4Addr::LOCALHOST,0),(Ipv4Addr::LOCALHOST,32)]).0.is_empty());
    }
    #[test]
    fn validates_discovery_replies() {
        let peer="192.168.1.101:27183".parse().unwrap();
        assert_eq!(response(b"QPO1\x2e\x6a\x01\x00",peer),Some("192.168.1.101:27182".into()));
        for invalid in [b"QPO1\x00\x00\x01\x00".as_slice(),b"QPO1\x2e\x6a\x02\x00",b"QPO1",b"QPX1\x2e\x6a\x01\x00"] {assert!(response(invalid,peer).is_none())}
    }
    #[test]
    fn cancellation_interrupts_search() {
        assert_eq!(discover(||true).unwrap_err().kind(),io::ErrorKind::Interrupted);
    }
    #[test]
    fn direct_udp_finds_module_without_broadcast_and_ignores_invalid_reply() {
        let module=UdpSocket::bind("127.0.0.1:0").unwrap();
        module.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let port=module.local_addr().unwrap().port();
        let responder=std::thread::spawn(move|| {
            let mut packet=[0u8;64];let (n,peer)=module.recv_from(&mut packet).unwrap();
            assert_eq!(&packet[..n],b"QPD1");
            module.send_to(b"invalid",peer).unwrap();
            module.send_to(b"QPO1\x2e\x6a\x01\x00",peer).unwrap();
        });
        let socket=UdpSocket::bind("127.0.0.1:0").unwrap();
        socket.set_read_timeout(Some(Duration::from_millis(25))).unwrap();
        assert_eq!(search(&socket,&[Ipv4Addr::LOCALHOST],&[],port,||false).unwrap(),"127.0.0.1:27182");
        responder.join().unwrap();
    }
}
