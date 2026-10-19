//! Synthetic hardware for isolated UI checks; no VM or host mutation is available.
use super::model::Detection;
use crate::config::{Boot, Config, Disk, DiskMode, Gpu, Hooks, Vm, Windows};
use crate::detect::{
    disks::{Esp, WindowsInstall},
    gpus,
    system::System,
};
use crate::setup::Inventory;

pub(super) fn detection() -> Detection {
    let gpu = gpus::Gpu {
        address: "0000:01:00.0".into(),
        name: "NVIDIA GeForce RTX 4060 Laptop GPU".into(),
        ids: vec!["10de:28a0".into(), "10de:22be".into()],
        audio_address: Some("0000:01:00.1".into()),
        driver: Some("vfio-pci".into()),
        blocker: None,
    };
    Detection {
        config: None,
        inventory: Inventory {
            windows: vec![WindowsInstall {
                disk: "/dev/nvme0n1".into(),
                disk_model: "Windows NVMe drive".into(),
                disk_id: Some("fixture-drive".into()),
                disk_guid: "fixture-guid".into(),
                disk_size: 1024_u64.pow(4),
                partition: "/dev/nvme0n1p3".into(),
                size: 950 * 1024_u64.pow(3),
                filesystem_uuid: "fixture-windows".into(),
                part_uuid: "fixture-partition".into(),
                bitlocker: false,
                esp: Some(Esp {
                    path: "/dev/nvme0n1p1".into(),
                    part_uuid: "fixture-esp".into(),
                    mounts: vec![],
                }),
                mounts: vec![],
                shares_linux: false,
                mode: DiskMode::Whole,
            }],
            gpus: vec![gpu],
            system: System {
                cpus: (0..20).collect(),
                performance_cpus: (0..12).collect(),
                efficiency_cpus: (12..20).collect(),
                threads_per_core: 2,
                memory_mib: 32768,
                iommu_enabled: true,
                iommu_flag: "intel_iommu=on",
                limine_kernel: Some("linux".into()),
                refresh_hz: "165".into(),
                user: "fixture".into(),
                omarchy: true,
                missing: vec![],
            },
        },
    }
}

pub(super) fn plan(choices: &crate::setup::Choices) -> crate::setup::Plan {
    crate::setup::Plan {
        config: Config {
            vm: Vm { name: "window-side".into(), memory_mib: choices.memory_mib, vcpus: choices.vcpus, virtio: false },
            disk: Disk { mode: DiskMode::Whole, id: "fixture-drive".into(), windows_partition_uuid: "fixture-windows".into() },
            gpu: choices.gpu.as_ref().map(|address| Gpu { address: address.clone(), audio_address: Some("0000:01:00.1".into()),
                ids: vec!["10de:28a0".into()] }),
            windows: Windows { provisioned: false, user: String::new(), refresh_hz: "165".into() }, boot: Boot::default(), hooks: Hooks::default(),
        },
        domain_xml: String::new(), plugin: choices.plugin,
        root: crate::setup::root::RootPlan { user: "fixture".into(), packages: vec![], groups: vec![], files: vec![], virtual_disk: None, gpu_kernel: None },
        notes: choices.gpu.as_ref().map(|_| "The GPU profile reserves the selected GPU for Windows until Linux restarts in the normal profile.".into()).into_iter().collect(),
    }
}


pub(super) fn first_boot(mode: &str) -> crate::first_boot::Status {
    use crate::first_boot::{Stage, Status};
    let stage = match mode {
        "drivers" => Stage::Drivers,
        "sign-in" => Stage::SignIn,
        "ready" => Stage::Ready,
        _ => Stage::Start,
    };
    Status {
        environment: "isolated-omabox".into(), domain: "fixture".into(), stage,
        storage_ready: Some(matches!(stage, Stage::Ready | Stage::SignIn)),
        network_ready: Some(matches!(stage, Stage::Ready | Stage::SignIn)),
    }
}


/// The configured machine behind the status fixtures: GPU passthrough on a dedicated drive.
pub(super) fn config() -> Config {
    Config {
        vm: Vm { name: "window-side".into(), memory_mib: 16384, vcpus: 10, virtio: true },
        disk: Disk { mode: DiskMode::Whole, id: "fixture-drive".into(), windows_partition_uuid: "fixture-windows".into() },
        gpu: Some(Gpu { address: "0000:01:00.0".into(), audio_address: Some("0000:01:00.1".into()), ids: vec!["10de:28a0".into()] }),
        windows: Windows { provisioned: true, user: "fixture".into(), refresh_hz: "165".into() },
        boot: Boot::default(),
        hooks: Hooks::default(),
    }
}
