// Other system libraries
pub fn get_pci_all_info() {
    use pci_info::PciInfo;
    let info = PciInfo::enumerate_pci().unwrap();
    for r in info {
        match r {
            Ok(device) => println!("{device:?}"),
            Err(error) => eprintln!("{error}"),
        }
    }
}
