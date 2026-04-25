import { LSPClient } from "@/utils/lsp_client";
import { V86, type V86Image } from "../../v86";
import WASM_PATH from "../../v86/build/v86.wasm?url";
import BIOS_URL from "../../v86/bios/seabios.bin?url";
import VGA_BIOS_URL from "../../v86/bios/vgabios.bin?url";
import FILE_SYSTEM_URL from "../../v86-image-build/alpine/images/alpine-fs.json?url";
import INIT_STATE from "../../v86-image-build/alpine/images/alpine-state.bin.txt?url";

const FILE_SYSTEM_FILE = import.meta.env.BASE_URL + "../../v86-image-build/alpine/images/alpine-rootfs-flat";
const BIOS_IMAGE = { url: BIOS_URL } as unknown as V86Image;
const VGA_BIOS_IMAGE = { url: VGA_BIOS_URL } as unknown as V86Image;
const INIT_STATE_IMAGE = { url: INIT_STATE, async: false } as unknown as V86Image;


export async function startVM(target: HTMLElement | null) {
    const instance = new V86({
        wasm_path: WASM_PATH,
        bios: BIOS_IMAGE,
        vga_bios: VGA_BIOS_IMAGE,
        autostart: true,
        memory_size: 1 * 1024 * 1024 * 1024,
        vga_memory_size: 8 * 1024 * 1024,
        network_relay_url: "<UNUSED>",
        bzimage_initrd_from_filesystem: true,
        cmdline: "rw root=host9p rootfstype=9p rootflags=trans=virtio,cache=loose modules=virtio_pci tsc=reliable init_on_free=on",
        disable_keyboard: false,
        disable_mouse: false,
        serial_console: { type: "none" },
        virtio_console: { type: "none" },
        screen_container: target,
        filesystem: {
            baseurl: FILE_SYSTEM_FILE,
            basefs: FILE_SYSTEM_URL,
        },
        initial_state: INIT_STATE_IMAGE,
    });
    let client = new LSPClient(instance);
    const isReady = await client.waitForEmulatorReady();
    return { isReady, instance, client }
}

export async function startVMandlisten() {
    const vm = await startVM(null)
    const serialListener = (byte: number) => {
        vm.client.update(byte);
        const char = String.fromCharCode(byte);
    };

    vm.instance.add_listener("serial0-output-byte", serialListener);
    vm.instance.serial0_send("\r\n");
    return vm
}