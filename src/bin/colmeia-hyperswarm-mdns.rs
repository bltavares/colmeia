use colmeia_dat1_core::{parse, DatUrlResolution};
use colmeia_hyperswarm_mdns::{hash_to_domain, packet};

// 7e5998407b3d9dbb94db21ff50ad6f1b1d2c79e476fbaf9856c342eb4382e7f5
// pra
// b62ef6792a0e2f2c5be328593532415d80c0f52c // 32 bytes
// Derivacao igual ao dat1
fn main() {
    let parse_result = parse("7e5998407b3d9dbb94db21ff50ad6f1b1d2c79e476fbaf9856c342eb4382e7f5")
        .expect("could not parse data");

    if let DatUrlResolution::HashUrl(hash) = parse_result {
        let mdns_url = hash_to_domain(hash.discovery_key());
        println!("Parse result: {:?}", mdns_url);

        let mdns_packet_bytes = packet(&mdns_url);
        println!("Packet result: {:?}", mdns_packet_bytes);

        // Criar socket e enviar para Destination: 224.0.0.251
        let socket =
            colmeia_hyperswarm_mdns::socket::create_shared().expect("Could not create socket");
        println!("Socket: {:?}", socket);
        socket
            .send_to(
                &mdns_packet_bytes,
                *colmeia_hyperswarm_mdns::socket::MULTICAST_DESTINATION,
            )
            .expect("Could not send bytes");

        let mut buffer = [0; 512];
        socket
            .recv(&mut buffer)
            .expect("Could not receive packet");
        let debug_vec: Vec<_> = buffer.iter().collect();
        println!("Buffer response: {:?}", &debug_vec);
    } else {
        println!("this is a regular DNS")
    }
}
