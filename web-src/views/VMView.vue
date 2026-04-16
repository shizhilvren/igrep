<template>
    <div class="vm-page">
        <header class="vm-toolbar">
            <button class="vm-btn" :disabled="running" @click="startVm">Start</button>
            <button class="vm-btn" :disabled="!running" @click="stopVm">Stop</button>
            <button class="vm-btn" :disabled="!running" @click="restartVm">Restart</button>
            <span class="vm-status">{{ status }}</span>
        </header>

        <section class="vm-serial-panel">
            <div class="vm-serial-controls">
                <input v-model="serialInput" class="vm-serial-input" type="text"
                    placeholder="Send command to serial (Enter to send)" :disabled="!running"
                    @keydown.enter.prevent="sendSerialInput" />
                <button class="vm-btn" :disabled="!running || !serialInput" @click="sendSerialInput">Send</button>
                <button class="vm-btn" :disabled="!serialLog" @click="clearSerialLog">Clear Log</button>
            </div>
            <pre class="vm-serial-log">{{ serialLog }}</pre>
        </section>

        <section class="vm-stage" ref="screenContainer">
            <div style="white-space: pre; font: 14px monospace; line-height: 14px"></div>
            <canvas style="display: none"></canvas>
        </section>
    </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { V86, type V86Image } from "../../v86";

import WASM_PATH from "../../v86/build/v86.wasm?url";
import BIOS_URL from "../../v86/bios/seabios.bin?url";
import VGA_BIOS_URL from "../../v86/bios/vgabios.bin?url";
import FREEDOS_URL from "../../VM/freedos722.img?url";
import ALPINE_URL from "../../VM/alpine_v86_edge_x86.bin?url";

const BIOS_IMAGE = { url: BIOS_URL } as unknown as V86Image;
const VGA_BIOS_IMAGE = { url: VGA_BIOS_URL } as unknown as V86Image;
const FREEDOS_IMAGE = { url: FREEDOS_URL } as unknown as V86Image;
const ALPINE_IMAGE = { url: ALPINE_URL, async: false } as unknown as V86Image;

const status = ref("Loading...");
const running = ref(false);
const screenContainer = ref<HTMLElement | null>(null);
const serialLog = ref("");
const serialInput = ref("");
const MAX_SERIAL_LOG_CHARS = 20000;

let disposed = false;
let emulator: V86 | null = null;
let serialBuffer = "";
let serialListener: ((byte: number) => void) | null = null;

function appendSerialLog(text: string): void {
    serialLog.value = `${serialLog.value}${text}`.slice(-MAX_SERIAL_LOG_CHARS);
}

function sendSerialInput(): void {
    if (!emulator || !serialInput.value) {
        return;
    }

    const payload = serialInput.value.endsWith("\n") ? serialInput.value : `${serialInput.value}\n`;
    emulator.serial0_send(payload);
    appendSerialLog(`\n$ ${serialInput.value}\n`);
    serialInput.value = "";
}

function clearSerialLog(): void {
    serialLog.value = "";
}

function clearScreen(): void {
    const target = screenContainer.value;
    if (!target) {
        return;
    }
    while (target.firstChild) {
        target.removeChild(target.firstChild);
    }
}

async function startVm(): Promise<void> {
    const target = screenContainer.value;
    if (!target || disposed) {
        return;
    }

    try {
        status.value = "Loading v86 runtime...";

        if (emulator) {
            void emulator.stop();
            emulator = null;
        }

        clearScreen();
        status.value = "Booting FreeDOS...";
        serialBuffer = "";
        serialLog.value = "";
        serialInput.value = "";
        emulator = new V86({
            wasm_path: WASM_PATH,
            memory_size: 1 * 1024 * 1024 * 1024,
            vga_memory_size: 8 * 1024 * 1024,
            bios: BIOS_IMAGE,
            vga_bios: VGA_BIOS_IMAGE,
            // fda: FREEDOS_IMAGE,
            cdrom: ALPINE_IMAGE,
            autostart: true,
            disable_keyboard: false,
            disable_mouse: false,
            serial_console: { type: "none" },
            virtio_console: { type: "none" },
            screen_container: target,
            filesystem: {},
            cmdline: "tsc=reliable mitigations=off random.trust_cpu=on",
        });

        if (disposed && emulator) {
            void emulator.stop();
            emulator = null;
            return;
        }

        running.value = true;
        status.value = "Running FreeDOS";

        const stages = [
            {
                test: "virt-custom login: ",
                send: "root\n",
            },
            {
                test: "virt-custom:~# \x1b[6n",
                send: "clangd --version\n",
            },
        ];
        let stage = 0;

        serialListener = (byte: number) => {
            const char = String.fromCharCode(byte);
            if (char === "\r") {
                return;
            }

            serialBuffer += char;
            appendSerialLog(char);

            // console.log("Received char:", char, "Current data:", serialBuffer);

            const current = stages[stage];

            if (!current) {
                return;
            }

            if (serialBuffer.endsWith(current.test)) {
                stage++;
                emulator?.serial0_send(current.send);

                const log = "Sending: " + current.send.replace(/\n/g, "\\n") + "\n";
                // console.log(log);
                // appendSerialLog(`\n${log}`);
            }
        };

        emulator.add_listener("serial0-output-byte", serialListener);
    } catch (error) {
        running.value = false;
        status.value = "Failed to start VM";
        console.error("v86 initialization failed", error);
    }
}

function stopVm(): void {
    if (!emulator) {
        return;
    }

    if (serialListener) {
        emulator.remove_listener("serial0-output-byte", serialListener);
        serialListener = null;
    }

    void emulator.stop();
    emulator = null;
    running.value = false;
    status.value = "Stopped";
}

function restartVm(): void {
    if (emulator) {
        emulator.restart();
        status.value = "Restarted";
        return;
    }
    void startVm();
}

onMounted(() => {
    disposed = false;
    void startVm();
});

onUnmounted(() => {
    disposed = true;
    stopVm();
});
</script>

<style scoped>
.vm-page {
    height: 100%;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.vm-toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-radius: 10px;
    background: linear-gradient(90deg, #1f2833, #2e3b4e);
}

.vm-btn {
    border: 1px solid #4f627a;
    border-radius: 8px;
    background: #0f1722;
    color: #c8d3e0;
    padding: 6px 10px;
    cursor: pointer;
}

.vm-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
}

.vm-status {
    margin-left: auto;
    color: #d8e0ea;
    font-size: 13px;
}

.vm-stage {
    flex: 1;
    min-height: 0;
    border-radius: 10px;
    overflow: hidden;
    background: #000;
    border: 1px solid #28374b;
}

.vm-stage :deep(canvas) {
    width: 100%;
    height: 100%;
    display: block;
}

.vm-serial-panel {
    border-radius: 10px;
    border: 1px solid #28374b;
    background: #0b1118;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.vm-serial-controls {
    display: flex;
    gap: 8px;
}

.vm-serial-input {
    flex: 1;
    min-width: 0;
    border: 1px solid #4f627a;
    border-radius: 8px;
    background: #0f1722;
    color: #c8d3e0;
    padding: 6px 10px;
}

.vm-serial-log {
    margin: 0;
    max-height: 160px;
    overflow: auto;
    border: 1px solid #1d2a39;
    border-radius: 8px;
    padding: 8px;
    font: 12px/1.35 monospace;
    color: #a9d3b4;
    background: #05080d;
    white-space: pre-wrap;
}
</style>