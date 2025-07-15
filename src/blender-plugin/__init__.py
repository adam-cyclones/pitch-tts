bl_info = {
    "name": "Text-to-Face Lipsync Generator",
    "blender": (4, 4, 3),
    "category": "Animation",
    "author": "Adam Crockett",
    "description": "Generate and import text-to-face lipsync JSON and audio using the Rust backend."
}

import bpy
import json
import os
import tempfile
import subprocess
import shutil
import platform
from typing import Optional, List, Dict, Any

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
        result = runner.run_command(["text-to-face", "list", "--json"])
        voices_json = json.loads(result.stdout)
        # voices_json is an array of objects: {id, display_name, ...}
        return [(v["id"], v["display_name"], "") for v in voices_json]
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

class TEXTTOFACE_PT_panel(bpy.types.Panel):
    bl_label = "Text-to-Face Lipsync"
    bl_idname = "TEXTTOFACE_PT_panel"
    bl_space_type = "VIEW_3D"
    bl_region_type = "UI"
    bl_category = "Text-to-Face"

    def draw(self, context):
        layout = self.layout
        props = context.scene.texttoface_props
        layout.prop(props, "text")
        layout.prop(props, "voice")
        layout.prop(props, "pitch")
        layout.operator(TEXTTOFACE_OT_preview_audio.bl_idname, text="Preview Audio")

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

def register():
    bpy.utils.register_class(TEXTTOFACE_AddonPreferences)
    bpy.utils.register_class(TEXTTOFACE_Props)
    bpy.types.Scene.texttoface_props = bpy.props.PointerProperty(type=TEXTTOFACE_Props)
    bpy.utils.register_class(TEXTTOFACE_PT_panel)
    bpy.utils.register_class(TEXTTOFACE_OT_preview_audio)

def unregister():
    bpy.utils.unregister_class(TEXTTOFACE_AddonPreferences)
    bpy.utils.unregister_class(TEXTTOFACE_PT_panel)
    del bpy.types.Scene.texttoface_props
    bpy.utils.unregister_class(TEXTTOFACE_Props)
    bpy.utils.unregister_class(TEXTTOFACE_OT_preview_audio) 