set QEMU="c:\Program Files\qemu\qemu-system-x86_64w.exe"

set VM="C:\Users\v0idnull\Documents\VMs\WinDev2004Eval\WinDev2004Eval-disk001.qcow2"

%QEMU% -accel hax -net nic,model=virtio -vga std -hda %VM% -net user -m 4096M -monitor stdio -usb -device usb-tablet -rtc base=localtime,clock=host -smp cores=1
