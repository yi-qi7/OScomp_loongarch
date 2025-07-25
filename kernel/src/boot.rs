use super::main;

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; 4096 * 16] = [0; 4096 * 16];

macro_rules! init_dwm {
    () => {
        "
        # 设置异常入口 0x180
        ori         $t0, $zero, 0x1     # CSR_DMW1_PLV0
        lu52i.d     $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
        csrwr       $t0, 0x180          # LOONGARCH_CSR_DMWIN0

        # 配置异常状态寄存器0x181
        ori         $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
        lu52i.d     $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
        csrwr       $t0, 0x181          # LOONGARCH_CSR_DMWIN1

        // addi.d    $t0, $zero,0x11
        // csrwr     $t0, 0x181               # LOONGARCH_CSR_DMWIN1
        "
    };
}
const BOOT_STACK_SIZE: usize = 4096 * 16;
/// The earliest entry point for the primary CPU.
///
/// We can't use bl to jump to higher address, so we use jirl to jump to higher
/// address.
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        init_dwm!(),
        "# Enable PG  初始化特权模式相关寄存器
        li.w        $t0, 0xb0       # PLV=0, IE=0, PG=1
        csrwr       $t0, 0x0        # LOONGARCH_CSR_CRMD   当前模式配置
        li.w        $t0, 0x00       # PLV=0, PIE=0, PWE=0
        csrwr       $t0, 0x1        # LOONGARCH_CSR_PRMD   先前模式配置
        li.w        $t0, 0x00       # FPE=0, SXE=0, ASXE=0, BTE=0
        csrwr       $t0, 0x2        # LOONGARCH_CSR_EUEN   扩展单元使能

        # 设置栈指针
        la.global   $sp, {boot_stack}
        li.d        $t0, {boot_stack_size}
        add.d       $sp, $sp, $t0       # setup boot stack
        csrrd       $a0, 0x20           # cpuid  读取处理器核心信息，作为main()的入参

        # 跳转到主函数
        la.global   $t0, {entry}
        jirl        $zero,$t0,0
        ",
        entry = sym main,
        boot_stack = sym BOOT_STACK,
        boot_stack_size = const BOOT_STACK_SIZE,
        options(noreturn),
    )
}


/* # 测试代码
        li.d    $t2, 0x80001fe001e0   # UART 虚拟地址 (DMWIN0映射)
        li.d    $t3, 'A'              # ASCII 'A'
        st.b    $t3, $t2, 0           # 写入THR寄存器 */