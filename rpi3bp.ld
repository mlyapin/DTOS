ENTRY(_start)

SECTIONS
{
    . = 0x80000;

    .text ALIGN(4K) : {
        *(.boot.text)
        *(.text*)
    }

    .rodata ALIGN(4K) : {
        *(.rodata*)
    }

    .data ALIGN(4K) : {
        *(.data*)
    }

    .bss ALIGN(4K) : {
         __bss_start = .;
         *(.bss*)
         __bss_end = .;
    }

    . = ALIGN(16);
    . = . + 0x4000;
    __bootstack_bottom = .;
}
