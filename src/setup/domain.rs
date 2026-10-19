//! Renders the libvirt domain from setup choices.
use std::fmt::Write;

pub struct Domain<'a> {
    pub name: &'a str,
    pub uuid: &'a str,
    pub mac: &'a str,
    pub memory_mib: u32,
    pub vcpus: u32,
    pub threads_per_core: u32,
    /// Host CPUs for each vCPU, then for QEMU's own threads; empty when unpinned.
    pub vcpu_pins: &'a [u32],
    pub emulator_pins: &'a [u32],
    pub disk: &'a str,
    pub serial: Option<&'a str>,
    /// Windows needs the VirtIO drivers before it can boot from a VirtIO disk.
    pub virtio: bool,
    pub looking_glass: bool,
}

fn ranges(cpus: &[u32]) -> String {
    let mut sorted = cpus.to_vec();
    sorted.sort_unstable();
    let mut parts: Vec<String> = Vec::new();
    let mut index = 0;
    while index < sorted.len() {
        let start = sorted[index];
        let mut end = start;
        while index + 1 < sorted.len() && sorted[index + 1] == end + 1 {
            index += 1;
            end = sorted[index];
        }
        parts.push(if start == end { start.to_string() } else { format!("{start}-{end}") });
        index += 1;
    }
    parts.join(",")
}

pub fn render(domain: &Domain) -> String {
    let threads = if domain.vcpus.is_multiple_of(domain.threads_per_core) { domain.threads_per_core } else { 1 };
    let cores = domain.vcpus / threads;
    let mut cputune = String::new();
    if domain.vcpu_pins.len() == domain.vcpus as usize && !domain.emulator_pins.is_empty() {
        let emulator = ranges(domain.emulator_pins);
        cputune.push_str("  <cputune>\n");
        for (vcpu, cpu) in domain.vcpu_pins.iter().enumerate() {
            let _ = writeln!(cputune, "    <vcpupin vcpu=\"{vcpu}\" cpuset=\"{cpu}\"/>");
        }
        let _ = writeln!(cputune, "    <emulatorpin cpuset=\"{emulator}\"/>\n    <iothreadpin iothread=\"1\" cpuset=\"{emulator}\"/>");
        cputune.push_str("  </cputune>\n");
    }
    // One VirtIO queue per vCPU lets each guest CPU submit I/O without sharing a queue.
    let (bus, target, driver_extra, nic) = if domain.virtio {
        ("virtio", "vda", format!(" iothread=\"1\" queues=\"{}\"", domain.vcpus.min(16)), "virtio")
    } else {
        ("sata", "sda", String::new(), "e1000e")
    };
    let serial = domain.serial.map(|serial| format!("      <serial>{serial}</serial>\n")).unwrap_or_default();
    let shmem = if domain.looking_glass {
        "    <shmem name=\"looking-glass\">\n      <model type=\"ivshmem-plain\"/>\n      <size unit=\"M\">64</size>\n    </shmem>\n"
    } else {
        ""
    };
    format!(
        r#"<domain type="kvm">
  <name>{name}</name>
  <uuid>{uuid}</uuid>
  <title>{name}</title>
  <description>Dual-boot Windows managed by window-side.</description>
  <memory unit="MiB">{memory}</memory>
  <currentMemory unit="MiB">{memory}</currentMemory>
  <vcpu placement="static">{vcpus}</vcpu>
  <iothreads>1</iothreads>
{cputune}  <os firmware="efi">
    <firmware>
      <feature enabled="no" name="secure-boot"/>
      <feature enabled="no" name="enrolled-keys"/>
    </firmware>
    <type arch="x86_64" machine="q35">hvm</type>
    <bootmenu enable="yes" timeout="3000"/>
  </os>
  <features>
    <acpi/>
    <apic/>
    <hyperv>
      <relaxed state="on"/>
      <vapic state="on"/>
      <spinlocks state="on" retries="8191"/>
      <vpindex state="on"/>
      <runtime state="on"/>
      <synic state="on"/>
      <stimer state="on"/>
      <frequencies state="on"/>
      <tlbflush state="on"/>
      <ipi state="on"/>
    </hyperv>
    <vmport state="off"/>
    <smm state="on"/>
  </features>
  <cpu mode="host-passthrough" check="none" migratable="off">
    <topology sockets="1" dies="1" cores="{cores}" threads="{threads}"/>
    <!-- With VMX visible, Windows VBS can fail to start under KVM and boot loop with 0xc0000001. -->
    <feature policy="disable" name="vmx"/>
  </cpu>
  <!-- Windows dual-booted with Linux should keep the RTC in UTC (RealTimeIsUniversal). -->
  <clock offset="utc">
    <timer name="rtc" tickpolicy="catchup"/>
    <timer name="pit" tickpolicy="delay"/>
    <timer name="hpet" present="no"/>
    <timer name="hypervclock" present="yes"/>
  </clock>
  <on_poweroff>destroy</on_poweroff>
  <on_reboot>restart</on_reboot>
  <on_crash>destroy</on_crash>
  <pm>
    <suspend-to-mem enabled="no"/>
    <suspend-to-disk enabled="no"/>
  </pm>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
    <disk type="block" device="disk">
      <driver name="qemu" type="raw" cache="none" io="native" discard="unmap"{driver_extra}/>
      <source dev="{disk}"/>
      <target dev="{target}" bus="{bus}"/>
{serial}      <boot order="1"/>
    </disk>
    <interface type="network">
      <mac address="{mac}"/>
      <source network="default"/>
      <model type="{nic}"/>
    </interface>
    <graphics type="spice" autoport="yes">
      <listen type="none"/>
      <image compression="off"/>
      <streaming mode="off"/>
      <clipboard copypaste="yes"/>
      <filetransfer enable="yes"/>
    </graphics>
    <video>
      <model type="virtio" heads="1" primary="yes"/>
    </video>
{shmem}    <channel type="spicevmc">
      <target type="virtio" name="com.redhat.spice.0"/>
    </channel>
    <channel type="unix">
      <target type="virtio" name="org.qemu.guest_agent.0"/>
    </channel>
    <input type="tablet" bus="usb"/>
    <input type="keyboard" bus="usb"/>
    <sound model="ich9"/>
    <audio id="1" type="spice"/>
    <redirdev bus="usb" type="spicevmc"/>
    <redirdev bus="usb" type="spicevmc"/>
    <tpm model="tpm-crb">
      <backend type="emulator" version="2.0"/>
    </tpm>
    <memballoon model="virtio"/>
    <rng model="virtio">
      <backend model="random">/dev/urandom</backend>
    </rng>
  </devices>
</domain>
"#,
        name = domain.name,
        uuid = domain.uuid,
        memory = domain.memory_mib,
        vcpus = domain.vcpus,
        disk = domain.disk,
        mac = domain.mac,
    )
}

/// Reads `<uuid>` and the MAC from an existing domain so Windows keeps its activation and network identity.
pub fn identity(xml: &str) -> (Option<String>, Option<String>) {
    let between = |start: &str, end: &str| {
        let from = xml.find(start)? + start.len();
        let to = xml[from..].find(end)? + from;
        Some(xml[from..to].trim().to_owned())
    };
    (between("<uuid>", "</uuid>"), between("<mac address=\"", "\""))
}

pub fn existing_serial(xml: &str) -> Option<String> {
    let from = xml.find("<serial>")? + "<serial>".len();
    let to = xml[from..].find("</serial>")? + from;
    Some(xml[from..to].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn domain() -> Domain<'static> {
        Domain {
            name: "window-side", uuid: "abc", mac: "52:54:00:00:00:01", memory_mib: 16384, vcpus: 8,
            threads_per_core: 2, vcpu_pins: &[4, 5, 6, 7, 8, 9, 10, 11], emulator_pins: &[12, 13, 14, 15],
            disk: "/dev/mapper/window-side", serial: None, virtio: true, looking_glass: true,
        }
    }

    #[test]
    fn hybrid_pins_and_looking_glass_render() {
        let xml = render(&domain());
        assert!(xml.contains(r#"<vcpupin vcpu="7" cpuset="11"/>"#));
        assert!(xml.contains(r#"<emulatorpin cpuset="12-15"/>"#));
        assert!(xml.contains(r#"cores="4" threads="2""#));
        assert!(xml.contains(r#"<source dev="/dev/mapper/window-side"/>"#));
        assert!(xml.contains(r#"<target dev="vda" bus="virtio"/>"#));
        assert!(xml.contains("ivshmem-plain"));
    }

    #[test]
    fn first_boot_uses_sata_and_e1000e_without_pins() {
        let xml = render(&Domain { virtio: false, looking_glass: false, vcpu_pins: &[], ..domain() });
        assert!(xml.contains(r#"<target dev="sda" bus="sata"/>"#));
        assert!(xml.contains(r#"<model type="e1000e"/>"#));
        assert!(!xml.contains("cputune"));
        assert!(!xml.contains("ivshmem"));
    }

    #[test]
    fn existing_identity_and_serial_are_read() {
        let xml = render(&Domain { serial: Some("CT1000"), ..domain() });
        assert_eq!(identity(&xml), (Some("abc".into()), Some("52:54:00:00:00:01".into())));
        assert_eq!(existing_serial(&xml).as_deref(), Some("CT1000"));
        assert_eq!(ranges(&[3, 1, 2, 7]), "1-3,7");
    }
}
