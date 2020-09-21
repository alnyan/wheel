use yboot2_proto::{
    ProtoV1,
    KernelProtocol,
    v1::CMDLINE_SIZE,
    MemoryMapInfo,
    ElfTables,
    VideoInfo,
    video::PixelFormat
};

global_asm!(".section .boot, \"wa\", %progbits");
#[no_mangle]
#[link_section = ".boot"]
pub static BOOT_DATA: ProtoV1 = ProtoV1 {
    hdr:                yboot2_proto::Header {
        kernel_magic: ProtoV1::KERNEL_MAGIC,
        loader_magic: [0; 8]
    },

    flags: yboot2_proto::FLAG_UPPER,

    elf_tables: ElfTables {
        strtab_hdr: 0,
        strtab_data: 0,
        symtab_hdr: 0,
        symtab_data: 0
    },
    memory_map: MemoryMapInfo::by_loader(),
    video: VideoInfo {
        width: 640,
        height: 480,
        format: PixelFormat::LfbBgr32,
        framebuffer: 0,
        pitch: 0
    },

    rsdp: 0,

    initrd_base: 0,
    initrd_size: 0,

    cmdline: [0; CMDLINE_SIZE]
};
