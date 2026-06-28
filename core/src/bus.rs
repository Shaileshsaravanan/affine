/* 
memory bus for the emulator

the address ranges:

==> BIOS:          0x0000_0000 ..= 0x0000_3FFF      (16 KiB)
==> EWRAM:         0x0200_0000 ..= 0x0203_FFFF      (256 KiB)
==> IWRAM:         0x0300_0000 ..= 0x0300_7FFF      (32 KiB)
==> I/O registers: 0x0400_0000 ..= 0x0400_03FF      (1 KiB)
==> Palette RAM:   0x0500_0000 ..= 0x0500_03FF      (1 KiB)
==> VRAM:          0x0600_0000 ..= 0x0601_7FFF      (96 KiB)
==> OAM:           0x0700_0000 ..= 0x0700_03FF      (1 KiB)
==> Cartridge ROM: 0x0800_0000 ..= 0xFFFF_FFFF      (up to 32 MiB)
==> Cartridge SRAM/Flash: 0x0E00_0000 ..= 0x0E00_FFFF (64 KiB)

unmapped address behaves as “open bus”: reads return 0, writes are ignored;
regions can be accessed beyond their physical size;
*/


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Region {

    Bios,
    Ewram,
    IWram,
    Io,
    Palette,
    Vram,
    Oam,
    CartSram,
    Rom

}

pub const BIOS_SIZE: u32 = 0x4000;
pub const EWRAM_SIZE: u32 = 0x10000;
pub const IWRAM_SIZE: u32 = 0x8000;
pub const IO_SIZE: u32 = 0x400;
pub const PALETTE_SIZE: u32 = 0x400;
pub const VRAM_SIZE: u32 = 0x18000;
pub const OAM_SIZE: u32 = 0x400;
pub const CART_SRAM_SIZE: u32 = 0x10000;

pub struct Bus {

    bios: Vec<u8>,
    ewram: Vec<u8>,
    iwram: Vec<u8>,
    io_regs: Vec<u8>,
    palette: Vec<u8>,
    vram: Vec<u8>,
    oam: Vec<u8>,
    rom: Vec<u8>,
    cart_sram: Vec<u8>,

}

impl Bus {

    pub fn new() -> Self {
        Self{

            bios: vec![0; BIOS_SIZE as usize],
            ewram: vec![0; EWRAM_SIZE as usize],
            iwram: vec![0; IWRAM_SIZE as usize],
            io_regs: vec![0; IO_SIZE as usize],
            palette: vec![0; PALETTE_SIZE as usize],
            vram: vec![0; VRAM_SIZE as usize],
            oam: vec![0; OAM_SIZE as usize],
            rom: Vec::new(),
            cart_sram: vec![0; CART_SRAM_SIZE as usize],

        }
    }

    pub fn load_rom(&mut self, data: &[u8]){
        self.rom = data.to_vec();
    }

    fn map_addr(&self, addr: u32) -> Option<(Region, usize)> {

        fn offset(start: u32, size: u32, addr: u32) -> usize {
            ((addr - start) % size) as usize
        }

        let regions = [
            (0x00, 0x0000_0000, BIOS_SIZE, Region::Bios),
            (0x02, 0x0200_0000, EWRAM_SIZE, Region::Ewram),
            (0x03, 0x0300_0000, IWRAM_SIZE, Region::IWram),
            (0x04, 0x0400_0000, IO_SIZE, Region::Io),
            (0x05, 0x0500_0000, PALETTE_SIZE, Region::Palette),
            (0x06, 0x0600_0000, VRAM_SIZE, Region::Vram),
            (0x07, 0x0700_0000, OAM_SIZE, Region::Oam),
            (0x0E, 0x0E00_0000, CART_SRAM_SIZE, Region::CartSram),
        ];

        let key = ((addr >> 24) & 0xFF) as u8;
        for (top, start, size, region) in regions.iter() {
            if key == *top {
                let off = offset(*start, *size, addr);
                return Some((*region, off));
            }
        }

        const ROM_START: u32 = 0x0800_0000;
        if addr >= ROM_START {
            return Some((Region::Rom, (addr - ROM_START) as usize));
        }

        None

    }

    pub fn read8(&self, addr: u32) -> u8 {

        if let Some((reg, off)) = self.map_addr(addr) {
            match reg {
                Region::Bios => self.bios[off],
                Region::Ewram => self.ewram[off],
                Region::IWram => self.iwram[off],
                Region::Io => self.io_regs[off],
                Region::Palette => self.palette[off],
                Region::Vram => self.vram[off],
                Region::Oam => self.oam[off],
                Region::Rom => {

                    if off < self.rom.len() {
                        self.rom[off]
                    } else {
                        0
                    }

                }
                Region::CartSram => self.cart_sram[off],
            }

        } else {
            0
        }
    }

    pub fn read16(&self, addr: u32) -> u16 {

        let lo = self.read8(addr);
        let hi = self.read8(addr + 1);
        (( hi as u16) << 8) | lo as u16

    }

    pub fn read32(&self, addr: u32) -> u32 {

        let b0 = self.read8(addr);
        let b1 = self.read8(addr + 1);
        let b2 = self.read8(addr + 2);
        let b3 = self.read8(addr + 3);
        ((b3 as u32) << 24) | ((b2 as u32) << 16) | ((b1 as u32) << 8) | b0 as u32

    }

    pub fn write8(&mut self, addr: u32, val: u8) {

        if let Some((reg, off)) = self.map_addr(addr) {
            match reg {

                Region::Bios => self.bios[off] = val,
                Region::Ewram => self.ewram[off] = val,
                Region::IWram => self.iwram[off] = val,
                Region::Io => self.io_regs[off] = val,
                Region::Palette => self.palette[off] = val,
                Region::Vram => self.vram[off] = val,
                Region::Oam => self.oam[off] = val,
                Region::Rom => {
                    if off < self.rom.len() {
                        self.rom[off] = val;
                    }
                }
                Region::CartSram => self.cart_sram[off] = val,

            }
        }

    }

    pub fn write16(&mut self, addr: u32, val: u16) {

        let lo = (val & 0xFF) as u8;
        let hi = ((val >> 8) & 0xFF) as u8;

        self.write8(addr, lo);
        self.write8(addr + 1, hi);

    }

    pub fn write32(&mut self, addr: u32, val:u32) {

        let b0 = (val & 0xFF) as u8;
        let b1 = ((val >> 8) & 0xFF) as u8;
        let b2 = ((val >> 16) & 0xFF) as u8;
        let b3 = ((val >> 24) & 0xFF) as u8;

        self.write8(addr, b0);
        self.write8(addr + 1, b1);
        self.write8(addr + 2, b2);
        self.write8(addr + 3, b3);

    }

}

#[cfg(test)]
mod tests {

    use super::Bus;

    #[test]
    fn ewram_write_read() {

        let mut bus = Bus::new();
        let addr = 0x0200_1000;

        bus.write8(addr, 0xA3);
        assert_eq!(bus.read8(addr), 0xA3);

    }

    #[test]
    fn iwram_write32_read32() {

        let mut bus = Bus::new();
        let addr = 0x0300_2000;
        let val = 0x11_22_33_44;

        bus.write32(addr, val);
        assert_eq!(bus.read32(addr), val);
        assert_eq!(bus.read8(addr), 0x44);
        assert_eq!(bus.read8(addr + 1), 0x33);
        assert_eq!(bus.read8(addr + 2), 0x22);
        assert_eq!(bus.read8(addr + 3), 0x11);

    }

    #[test]
    fn load_rom_read_first_byte() {

        let mut bus = Bus::new();
        let rom_data = vec![0xDE, 0xAD, 0xBE, 0xEF];

        bus.load_rom(&rom_data);
        assert_eq!(bus.read8(0x0800_0000), 0xDE);
        assert_eq!(bus.read8(0x0800_0001), 0xAD);

    }

    #[test]
    fn iwram_mirror() {

        let mut bus = Bus::new();
        let mirrored_addr = 0x0300_8000;

        bus.write8(mirrored_addr, 0x7C);
        assert_eq!(bus.read8(0x0300_0000), 0x7C);

    }

    #[test]
    fn open_bus_read_return_zero() {

        let bus = Bus::new();
        
        assert_eq!(bus.read8(0x0A00_0000), 0);
        assert_eq!(bus.read16(0x0B00_0000), 0);
        assert_eq!(bus.read32(0x0C00_0000), 0);

    }

    #[test]
    fn cart_sram_write_read() { 

        let mut bus = Bus::new();
        let addr = 0x0E00_0100;

        bus.write8(addr, 0x9A);
        assert_eq!(bus.read8(addr), 0x9A);

    }

}