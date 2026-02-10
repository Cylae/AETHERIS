use aetheris_core::services::arr::{LidarrService, ReadarrService};
use aetheris_core::services::Service;

#[test]
fn test_lidarr_service_definition() {
    let service = LidarrService;
    assert_eq!(service.name(), "lidarr");
    assert_eq!(service.image(), "lscr.io/linuxserver/lidarr:latest");

    let ports = service.ports();
    assert_eq!(ports.len(), 1);
    assert!(ports[0].contains("8686"));
}

#[test]
fn test_readarr_service_definition() {
    let service = ReadarrService;
    assert_eq!(service.name(), "readarr");
    assert_eq!(service.image(), "lscr.io/linuxserver/readarr:latest");

    let ports = service.ports();
    assert_eq!(ports.len(), 1);
    assert!(ports[0].contains("8787"));
}
