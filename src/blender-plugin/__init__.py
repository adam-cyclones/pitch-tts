bl_info = {
    "name": "Text-to-Face Lipsync Generator",
    "blender": (4, 4, 3),
    "category": "Animation",
    "author": "Your Name",
    "description": "Generate and import text-to-face lipsync JSON and audio using the Rust backend.",
    "version": (0, 1, 2),
}

import bpy
import json
import os
import tempfile
import subprocess
import shutil
import platform
import hashlib
from typing import Optional, List, Dict, Any
import math
from bpy_extras.io_utils import ImportHelper

# Simple ARPAbet to viseme mapping (customize as needed)
ARPABET_TO_VISEME = {
    "AA": "viseme_AA", "AE": "viseme_AA", "AH": "viseme_AA", "AO": "viseme_AA",
    "AW": "viseme_AA", "AY": "viseme_AA", "B": "viseme_BM", "M": "viseme_BM", "P": "viseme_BM",
    "CH": "viseme_CH", "JH": "viseme_CH", "D": "viseme_D", "DH": "viseme_D", "T": "viseme_D",
    "EH": "viseme_E", "EY": "viseme_E", "F": "viseme_FV", "V": "viseme_FV",
    "G": "viseme_GK", "K": "viseme_GK", "NG": "viseme_GK",
    "IH": "viseme_I", "IY": "viseme_I", "L": "viseme_L", "N": "viseme_N",
    "OW": "viseme_O", "OY": "viseme_O", "UH": "viseme_O", "UW": "viseme_O",
    "R": "viseme_R", "S": "viseme_S", "Z": "viseme_S", "SH": "viseme_SH", "ZH": "viseme_SH",
    "TH": "viseme_TH", "W": "viseme_W", "Y": "viseme_Y"
}

# Use the global CLI
CLI_COMMAND = "text-to-face"

# --- BlenderCommandRunner for robust CLI discovery ---
class BlenderCommandRunner:
    """Safe cross-platform command runner for Blender environments"""
    def __init__(self):
        self.system = platform.system()
        self.extended_env = self._get_extended_environment()
    def _get_shell_paths(self) -> List[str]:
        additional_paths = []
        if self.system == "Darwin":
            paths = [
                "/usr/local/bin",
                "/opt/homebrew/bin",
                "/opt/homebrew/sbin",
                "/usr/bin",
                "/bin",
                "/usr/sbin",
                "/sbin"
            ]
            shell_commands = [
                ["/bin/zsh", "-l", "-c", "echo $PATH"],
                ["/bin/bash", "-l", "-c", "echo $PATH"]
            ]
            for cmd in shell_commands:
                try:
                    result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
                    if result.returncode == 0:
                        shell_paths = result.stdout.strip().split(':')
                        paths.extend([p for p in shell_paths if p and p not in paths])
                        break
                except:
                    continue
            additional_paths.extend(paths)
        elif self.system == "Linux":
            paths = [
                "/usr/local/bin",
                "/usr/bin",
                "/bin",
                "/usr/sbin",
                "/sbin",
                "/snap/bin",
                "/usr/local/sbin"
            ]
            try:
                result = subprocess.run(["/bin/bash", "-l", "-c", "echo $PATH"], capture_output=True, text=True, timeout=5)
                if result.returncode == 0:
                    shell_paths = result.stdout.strip().split(':')
                    paths.extend([p for p in shell_paths if p and p not in paths])
            except:
                pass
            additional_paths.extend(paths)
        elif self.system == "Windows":
            paths = [
                r"C:\Windows\System32",
                r"C:\Windows",
                r"C:\Windows\System32\WindowsPowerShell\v1.0",
                r"C:\Program Files\Git\bin",
                r"C:\Program Files\Python39\Scripts",
                r"C:\Program Files\Python310\Scripts",
                r"C:\Program Files\Python311\Scripts",
                r"C:\Program Files\Python312\Scripts"
            ]
            additional_paths.extend(paths)
        home = os.path.expanduser("~")
        user_paths = [
            os.path.join(home, ".local", "bin"),
            os.path.join(home, "bin"),
            os.path.join(home, ".cargo", "bin"),
            os.path.join(home, "go", "bin"),
        ]
        if self.system == "Windows":
            user_paths.extend([
                os.path.join(home, "AppData", "Local", "Programs", "Python", "Python311", "Scripts"),
                os.path.join(home, "AppData", "Roaming", "Python", "Python311", "Scripts"),
            ])
        additional_paths.extend(user_paths)
        return [path for path in additional_paths if os.path.isdir(path)]
    def _get_extended_environment(self) -> Dict[str, str]:
        env = os.environ.copy()
        current_path = env.get('PATH', '')
        path_separator = ';' if self.system == "Windows" else ':'
        additional_paths = self._get_shell_paths()
        all_paths = additional_paths + [current_path] if current_path else additional_paths
        env['PATH'] = path_separator.join(all_paths)
        return env
    def find_command(self, command: str) -> Optional[str]:
        command_path = shutil.which(command, path=self.extended_env.get('PATH'))
        if command_path:
            return command_path
        command_path = shutil.which(command)
        if command_path:
            return command_path
        return None
    def run_command(self, command: List[str], timeout: int = 30, capture_output: bool = True, text: bool = True, check: bool = True) -> subprocess.CompletedProcess:
        if not command:
            raise ValueError("Command cannot be empty")
        command_path = self.find_command(command[0])
        if not command_path:
            raise FileNotFoundError(f"Command '{command[0]}' not found in PATH")
        full_command = [command_path] + command[1:]
        return subprocess.run(
            full_command,
            env=self.extended_env,
            timeout=timeout,
            capture_output=capture_output,
            text=text,
            check=check
        )
    def test_command(self, command: str, args: List[str] = None) -> bool:
        try:
            test_args = args or ["--version"]
            result = self.run_command([command] + test_args, timeout=10, capture_output=True, check=False)
            return result.returncode == 0
        except:
            return False

# --- Use the runner for all CLI calls ---
runner = BlenderCommandRunner()

def is_cli_available():
    return runner.test_command("text-to-face")

def get_app_data_dir():
    # Match the Rust ProjectDirs logic
    home = os.path.expanduser("~")
    if platform.system() == "Darwin":
        return os.path.join(home, "Library", "Application Support", "com.yourorg.text-to-face")
    elif platform.system() == "Linux":
        return os.path.join(home, ".local", "share", "com.yourorg.text-to-face")
    elif platform.system() == "Windows":
        return os.path.join(os.environ.get("APPDATA", os.path.join(home, "AppData", "Roaming")), "com.yourorg.text-to-face")
    else:
        return os.path.join(home, ".text-to-face")

def get_generated_wav_path(blend_filepath, voice_id, text):
    base = os.path.splitext(os.path.basename(blend_filepath))[0] if blend_filepath else "untitled"
    text_hash = hashlib.sha256(text.encode("utf-8")).hexdigest()[:8]
    app_data = get_app_data_dir()
    out_dir = os.path.join(app_data, "generated")
    os.makedirs(out_dir, exist_ok=True)
    filename = f"{base}_{voice_id}_{text_hash}.wav"
    return os.path.join(out_dir, filename)

class TEXTTOFACE_AddonPreferences(bpy.types.AddonPreferences):
    bl_idname = __name__

    def draw(self, context):
        layout = self.layout
        if is_cli_available():
            layout.label(text="Text-to-Face CLI found.", icon="CHECKMARK")
        else:
            layout.label(text="Text-to-Face CLI not found! Please install it and ensure it's in your PATH.", icon="ERROR")

def clear_viseme_keys(obj, frame_start, frame_end):
    if not obj.data.shape_keys:
        return
    for key in obj.data.shape_keys.key_blocks:
        if key.name.startswith("viseme_"):
            for f in range(int(frame_start), int(frame_end)+1):
                key.value = 0.0
                key.keyframe_insert(data_path="value", frame=f)

def insert_viseme(obj, viseme, frame, strength=1.0):
    if not obj.data.shape_keys:
        return
    if viseme in obj.data.shape_keys.key_blocks:
        key = obj.data.shape_keys.key_blocks[viseme]
        key.value = strength
        key.keyframe_insert(data_path="value", frame=frame)

def get_voices():
    try:
        # Get all voices
        result_all = runner.run_command(["text-to-face", "list", "--json"])
        all_voices = json.loads(result_all.stdout)
        # Get installed voices
        result_installed = runner.run_command(["text-to-face", "list", "--installed", "--json"])
        installed_voices = {v["id"] for v in json.loads(result_installed.stdout)}
        # Build dropdown: label as (installed) or (not installed)
        items = []
        for v in all_voices:
            label = v["display_name"]
            if v["id"] in installed_voices:
                label += " (installed)"
            items.append((v["id"], label, ""))
        return items
    except Exception as e:
        print(f"[Text-to-Face] Failed to get voices: {e}")
        return []

class TEXTTOFACE_OT_preview_audio(bpy.types.Operator):
    bl_idname = "texttoface.preview_audio"
    bl_label = "Preview Audio"
    bl_description = "Preview TTS audio using Rust backend"
    def execute(self, context):
        props = context.scene.texttoface_props
        if not props.text.strip():
            self.report({'ERROR'}, "Enter some text!")
            return {'CANCELLED'}
        if not props.voice:
            self.report({'ERROR'}, "Select a voice!")
            return {'CANCELLED'}
        # Check if selected voice is installed
        try:
            result_installed = runner.run_command(["text-to-face", "list", "--installed", "--json"])
            installed_voices = {v["id"] for v in json.loads(result_installed.stdout)}
            if props.voice not in installed_voices:
                self.report({'ERROR'}, "Selected voice is not installed. Please download it first.")
                return {'CANCELLED'}
        except Exception as e:
            self.report({'ERROR'}, f"Failed to check installed voices: {e}")
            return {'CANCELLED'}
        cmd = [
            "text-to-face", "say",
            props.text,
            "--voice", props.voice,
            "--pitch", str(props.pitch)
        ]
        try:
            runner.run_command(cmd)
            self.report({'INFO'}, "Audio previewed!")
        except Exception as e:
            self.report({'ERROR'}, f"Failed to preview audio: {e}")
            return {'CANCELLED'}
        return {'FINISHED'}

class TEXTTOFACE_OT_generate_audio(bpy.types.Operator):
    bl_idname = "texttoface.generate_audio"
    bl_label = "Generate Audio"
    bl_description = "Generate TTS audio, load it into Blender, and auto-split for lipsync"
    def execute(self, context):
        props = context.scene.texttoface_props
        if not props.text.strip():
            self.report({'ERROR'}, "Enter some text!")
            return {'CANCELLED'}
        if not props.voice:
            self.report({'ERROR'}, "Select a voice!")
            return {'CANCELLED'}
        blend_path = bpy.data.filepath
        wav_path = get_generated_wav_path(blend_path, props.voice, props.text)
        json_path = wav_path[:-4] + ".json"
        cmd = [
            "text-to-face", "export",
            props.text,
            "--voice", props.voice,
            "--output", wav_path,
            "--pitch", str(props.pitch)
        ]
        try:
            runner.run_command(cmd)
            # Add audio to the VSE timeline
            bpy.ops.sequencer.sound_strip_add(filepath=wav_path, frame_start=1, channel=1)
            # Find the loaded audio strip in the VSE
            seq = context.scene.sequence_editor
            if not seq:
                seq = context.scene.sequence_editor_create()
            audio_strip = None
            for s in seq.sequences_all:
                if s.type == 'SOUND' and s.sound and s.sound.filepath.endswith(os.path.basename(wav_path)):
                    audio_strip = s
                    break
            if not audio_strip:
                self.report({'ERROR'}, "Audio strip not found in VSE after loading.")
                return {'CANCELLED'}
            # Parse JSON for word/phoneme timings
            if not os.path.exists(json_path):
                self.report({'ERROR'}, f"JSON timing file not found: {json_path}")
                return {'CANCELLED'}
            with open(json_path, 'r') as f:
                lipsync_data = json.load(f)
            fps = context.scene.render.fps
            word_segments = lipsync_data.get('word_segments', [])
            # Split at word boundaries
            strips_by_word = []
            for i, word in enumerate(word_segments):
                word_start = int(round(word['start'] * fps))
                word_end = int(round(word['end'] * fps))
                if i == 0:
                    audio_strip.frame_final_start = word_start
                # Split at word_end (unless last word)
                if i < len(word_segments) - 1:
                    bpy.ops.sequencer.select_all(action='DESELECT')
                    audio_strip.select = True
                    context.scene.frame_current = word_end
                    bpy.ops.sequencer.split(frame=word_end, type='SOFT')
                    # After split, find the left strip (start to word_end)
                    for s in seq.sequences_all:
                        if s.type == 'SOUND' and s.frame_final_start == word_start and s.frame_final_end == word_end:
                            s.name = f"word_{word['word']}"
                            strips_by_word.append((s, word))
                            break
                    # Find the right strip for next iteration
                    for s in seq.sequences_all:
                        if s.type == 'SOUND' and s.frame_final_start == word_end:
                            audio_strip = s
                            break
                else:
                    # Last word
                    audio_strip.name = f"word_{word['word']}"
                    strips_by_word.append((audio_strip, word))
            # For each word, split into phoneme segments
            for strip, word in strips_by_word:
                phonemes = word.get('phonemes', [])
                n = len(phonemes)
                if n < 2:
                    strip.name = f"{strip.name}_ph_{phonemes[0] if phonemes else ''}"
                    continue
                word_start = strip.frame_final_start
                word_end = strip.frame_final_end
                length = word_end - word_start
                ph_frames = [word_start + int(round(i * length / n)) for i in range(1, n)]
                ph_strips = []
                prev_start = word_start
                for j, ph_frame in enumerate(ph_frames):
                    bpy.ops.sequencer.select_all(action='DESELECT')
                    strip.select = True
                    context.scene.frame_current = ph_frame
                    bpy.ops.sequencer.split(frame=ph_frame, type='SOFT')
                    # Find left strip
                    for s in seq.sequences_all:
                        if s.type == 'SOUND' and s.frame_final_start == prev_start and s.frame_final_end == ph_frame:
                            s.name = f"ph_{phonemes[j]}"
                            ph_strips.append(s)
                            break
                    # Find right strip for next iteration
                    for s in seq.sequences_all:
                        if s.type == 'SOUND' and s.frame_final_start == ph_frame:
                            strip = s
                            break
                    prev_start = ph_frame
                # Last phoneme
                strip.name = f"ph_{phonemes[-1]}"
                ph_strips.append(strip)
                # Optionally group phoneme strips into a Meta Strip
                bpy.ops.sequencer.select_all(action='DESELECT')
                for s in ph_strips:
                    s.select = True
                context.scene.frame_current = ph_strips[0].frame_final_start
                bpy.ops.sequencer.meta_make()
                meta = None
                for s in seq.sequences_all:
                    if s.type == 'META' and s.frame_final_start == ph_strips[0].frame_final_start and s.frame_final_end == ph_strips[-1].frame_final_end:
                        meta = s
                        break
                if meta:
                    meta.name = f"word_{word['word']}_meta"
            self.report({'INFO'}, f"Audio generated, loaded, and split for lipsync: {wav_path}")
        except Exception as e:
            self.report({'ERROR'}, f"Failed to generate/split audio: {e}")
            return {'CANCELLED'}
        return {'FINISHED'}

class TEXTTOFACE_OT_export_shape_keys(bpy.types.Operator):
    bl_idname = "texttoface.export_shape_keys"
    bl_label = "Export to Shape Keys"
    bl_description = "Export current phoneme strips to shape key keyframes on the selected object"
    def execute(self, context):
        obj = context.object
        if not obj or not obj.data.shape_keys:
            self.report({'ERROR'}, "Select a mesh object with shape keys!")
            return {'CANCELLED'}
        seq = context.scene.sequence_editor
        if not seq:
            self.report({'ERROR'}, "No sequence editor found!")
            return {'CANCELLED'}
        # Map ARPAbet to viseme/shape key
        mapping = ARPABET_TO_VISEME
        for s in seq.sequences_all:
            if s.type == 'SOUND' and s.name.startswith('ph_'):
                ph = s.name[3:]
                viseme = mapping.get(ph.replace('1','').replace('0',''), None)
                if viseme and viseme in obj.data.shape_keys.key_blocks:
                    # Insert keyframes at start and end
                    key = obj.data.shape_keys.key_blocks[viseme]
                    key.value = 1.0
                    key.keyframe_insert(data_path="value", frame=s.frame_final_start)
                    key.value = 0.0
                    key.keyframe_insert(data_path="value", frame=s.frame_final_end)
        self.report({'INFO'}, "Shape key keyframes exported from phoneme strips!")
        return {'FINISHED'}

class TEXTTOFACE_OT_load_last_audio(bpy.types.Operator):
    bl_idname = "texttoface.load_last_audio"
    bl_label = "Load Last Generated Audio"
    bl_description = "Load the last generated audio for this file/voice/text if it exists"
    def execute(self, context):
        props = context.scene.texttoface_props
        blend_path = bpy.data.filepath
        wav_path = get_generated_wav_path(blend_path, props.voice, props.text)
        if os.path.exists(wav_path):
            bpy.ops.sound.open(filepath=wav_path)
            self.report({'INFO'}, f"Loaded audio: {wav_path}")
            return {'FINISHED'}
        else:
            self.report({'ERROR'}, "No generated audio found for this file/voice/text.")
            return {'CANCELLED'}

class TEXTTOFACE_PT_panel(bpy.types.Panel):
    bl_label = "Text-to-Face Lipsync"
    bl_idname = "TEXTTOFACE_PT_panel"
    bl_space_type = "SEQUENCE_EDITOR"  # Moved from VIEW_3D to SEQUENCE_EDITOR
    bl_region_type = "UI"
    bl_category = "Text-to-Face"

    def draw(self, context):
        layout = self.layout
        props = context.scene.texttoface_props
        layout.prop(props, "text")
        layout.prop(props, "voice")
        layout.prop(props, "pitch")
        layout.operator(TEXTTOFACE_OT_preview_audio.bl_idname, text="Preview Audio")
        layout.operator(TEXTTOFACE_OT_generate_audio.bl_idname, text="Generate Audio and Split for Lipsync")
        layout.operator(TEXTTOFACE_OT_load_last_audio.bl_idname, text="Load Last Generated Audio")
        layout.operator(TEXTTOFACE_OT_export_shape_keys.bl_idname, text="Export to Shape Keys")
        # Placeholder for new lipsync workflow UI (phoneme/word splitting, keyframe export, etc.)

class TEXTTOFACE_Props(bpy.types.PropertyGroup):
    text: bpy.props.StringProperty(
        name="Text",
        description="Text to synthesize",
        default="Hello from Blender!"
    )
    voice: bpy.props.EnumProperty(
        name="Voice",
        description="Voice to use",
        items=lambda self, context: get_voices() or [("", "(No voices found)", "")],
    )
    pitch: bpy.props.FloatProperty(
        name="Pitch",
        description="Pitch factor (1.0 = normal)",
        default=1.0,
        min=0.5,
        max=2.0
    )

class TEXTTOFACE_PT_voice_library(bpy.types.Panel):
    bl_label = "Voice Library"
    bl_idname = "TEXTTOFACE_PT_voice_library"
    bl_space_type = "SEQUENCE_EDITOR"  # Moved from VIEW_3D to SEQUENCE_EDITOR
    bl_region_type = "UI"
    bl_category = "Text-to-Face"
    def draw(self, context):
        layout = self.layout
        app_data = get_app_data_dir()
        gen_dir = os.path.join(app_data, "generated")
        if not os.path.exists(gen_dir):
            layout.label(text="No generated audio found.")
            return
        wavs = [f for f in os.listdir(gen_dir) if f.endswith(".wav")]
        if not wavs:
            layout.label(text="No generated audio found.")
            return
        for wav in sorted(wavs):
            row = layout.row()
            row.label(text=wav)
            op = row.operator("texttoface.load_specific_audio", text="Load")
            op.wav_filename = wav

class TEXTTOFACE_OT_load_specific_audio(bpy.types.Operator):
    bl_idname = "texttoface.load_specific_audio"
    bl_label = "Load Audio"
    bl_description = "Load this generated audio into Blender"
    wav_filename: bpy.props.StringProperty()
    def execute(self, context):
        app_data = get_app_data_dir()
        wav_path = os.path.join(app_data, "generated", self.wav_filename)
        if os.path.exists(wav_path):
            bpy.ops.sound.open(filepath=wav_path)
            self.report({'INFO'}, f"Loaded audio: {wav_path}")
            return {'FINISHED'}
        else:
            self.report({'ERROR'}, "Audio file not found.")
            return {'CANCELLED'}

def register():
    bpy.utils.register_class(TEXTTOFACE_AddonPreferences)
    bpy.utils.register_class(TEXTTOFACE_Props)
    bpy.types.Scene.texttoface_props = bpy.props.PointerProperty(type=TEXTTOFACE_Props)
    bpy.utils.register_class(TEXTTOFACE_PT_panel)
    bpy.utils.register_class(TEXTTOFACE_PT_voice_library)
    bpy.utils.register_class(TEXTTOFACE_OT_preview_audio)
    bpy.utils.register_class(TEXTTOFACE_OT_generate_audio)
    bpy.utils.register_class(TEXTTOFACE_OT_load_last_audio)
    bpy.utils.register_class(TEXTTOFACE_OT_load_specific_audio)
    bpy.utils.register_class(TEXTTOFACE_OT_export_shape_keys)

def unregister():
    bpy.utils.unregister_class(TEXTTOFACE_AddonPreferences)
    bpy.utils.unregister_class(TEXTTOFACE_PT_panel)
    bpy.utils.unregister_class(TEXTTOFACE_PT_voice_library)
    bpy.utils.unregister_class(TEXTTOFACE_OT_preview_audio)
    bpy.utils.unregister_class(TEXTTOFACE_OT_generate_audio)
    bpy.utils.unregister_class(TEXTTOFACE_OT_load_last_audio)
    bpy.utils.unregister_class(TEXTTOFACE_OT_load_specific_audio)
    bpy.utils.unregister_class(TEXTTOFACE_OT_export_shape_keys)
    del bpy.types.Scene.texttoface_props
    bpy.utils.unregister_class(TEXTTOFACE_Props) 