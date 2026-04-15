<template>
    <div class="vm-page">
        <header class="vm-toolbar">
            <button class="vm-btn" :disabled="running" @click="startVm">Start</button>
            <button class="vm-btn" :disabled="!running" @click="stopVm">Stop</button>
            <button class="vm-btn" :disabled="!running" @click="restartVm">Restart</button>
            <span class="vm-status">{{ status }}</span>
        </header>

        <section class="vm-stage" ref="screenContainer">
            <div style="white-space: pre; font: 14px monospace; line-height: 14px"></div>
            <canvas style="display: none"></canvas>
        </section>
    </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { V86, type V86Image } from "../../v86";

const WASM_PATH = "/v86/build/v86.wasm";
const BIOS_URL = "/v86/bios/seabios.bin";
const VGA_BIOS_URL = "/v86/bios/vgabios.bin";
const FREEDOS_URL = "/VM/freedos722.img";

const BIOS_IMAGE = { url: BIOS_URL } as unknown as V86Image;
const VGA_BIOS_IMAGE = { url: VGA_BIOS_URL } as unknown as V86Image;
const FREEDOS_IMAGE = { url: FREEDOS_URL } as unknown as V86Image;

const status = ref("Loading...");
const running = ref(false);
const screenContainer = ref<HTMLElement | null>(null);

let disposed = false;
let emulator: V86 | null = null;

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
        emulator = new V86({
            wasm_path: WASM_PATH,
            memory_size: 5 * 1024 * 1024,
            vga_memory_size: 8 * 1024 * 1024,
            bios: BIOS_IMAGE,
            vga_bios: VGA_BIOS_IMAGE,
            fda: FREEDOS_IMAGE,
            autostart: true,
            disable_keyboard: false,
            disable_mouse: false,
            serial_console: { type: "none" },
            virtio_console: { type: "none" },
            screen_container: target,
        });

        if (disposed && emulator) {
            void emulator.stop();
            emulator = null;
            return;
        }

        running.value = true;
        status.value = "Running FreeDOS";
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
</style>