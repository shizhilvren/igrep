#!/usr/bin/env node

import path from "node:path";
import fs from "node:fs";
import url from "node:url";
import { V86 } from "../../v86/build/libv86.mjs";

console.log("Don't forget to run `make all` before running this script");

const __dirname = url.fileURLToPath(new URL(".", import.meta.url));

const V86_ROOT = path.join(__dirname, "../../v86");
const IMAGE_ROOT = path.join(__dirname, "./");
const OUTPUT_FILE = "images/alpine-state.bin.txt";
console.log(path.join(V86_ROOT, "build/v86.wasm"))

var emulator = new V86({
    wasm_path: path.join(V86_ROOT, "build/v86.wasm"),
    bios: { url: path.join(V86_ROOT, "bios/seabios.bin") },
    vga_bios: { url: path.join(V86_ROOT, "bios/vgabios.bin") },
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
    filesystem: {
        baseurl: path.join(IMAGE_ROOT, "images/alpine-rootfs-flat"),
        basefs: path.join(IMAGE_ROOT, "images/alpine-fs.json"),
    },

});

console.log("Now booting, please stand by ...");

let serial_text = "";
let booted = false;
let num = 0;


emulator.add_listener("serial0-output-byte", function (byte) {
    const c = String.fromCharCode(byte);
    //process.stdout.write(c);

    serial_text += c;
    console.log(serial_text)

    if (!booted && (serial_text.endsWith("(none):~# ") || serial_text.endsWith("stdin/stdout"))) {
        if (num == 0) {
            num = num + 1;
            emulator.serial0_send("which clangd\n");
        } else if (num == 1) {
            num = num + 1;
            emulator.serial0_send("clangd\n");
        } else {
            booted = true;
            console.log(serial_text)
            // emulator.serial0_send("sync;echo 3 >/proc/sys/vm/drop_caches\n");

            setTimeout(async function () {
                const s = await emulator.save_state();

                fs.writeFile(OUTPUT_FILE, new Uint8Array(s), function (e) {
                    if (e) throw e;
                    console.log("Saved as " + OUTPUT_FILE);
                    emulator.destroy();
                });
            }, 10 * 1000);
        }
    }
});
