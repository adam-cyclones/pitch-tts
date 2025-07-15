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
import subprocess
import tempfile

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

# Path to the bundled binary
BINARY_PATH = os.path.join(os.path.dirname(__file__), "text-to-face")

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
        result = subprocess.run([BINARY_PATH, "list", "--by-language"], capture_output=True, text=True, check=True)
        lines = result.stdout.splitlines()
        voices = []
        for line in lines:
            if " - " in line:
                parts = line.split(" - ")
                voice_id = parts[0].strip()
                voices.append(voice_id)
        return voices
    except Exception as e:
        print(f"[Text-to-Face] Failed to get voices: {e}")
        return []

class TEXTTOFACE_OT_generate_and_import(bpy.types.Operator):
    bl_idname = "texttoface.generate_and_import"
    bl_label = "Generate & Import Lipsync"
    bl_description = "Generate lipsync JSON/audio using Rust backend and import animation"

    def execute(self, context):
        props = context.scene.texttoface_props
        obj = context.object
        if not obj or not obj.data.shape_keys:
            self.report({'ERROR'}, "Select a mesh with shape keys!")
            return {'CANCELLED'}
        if not props.text.strip():
            self.report({'ERROR'}, "Enter some text!")
            return {'CANCELLED'}
        if not props.voice:
            self.report({'ERROR'}, "Select a voice!")
            return {'CANCELLED'}
        tmpdir = os.path.join(tempfile.gettempdir(), "text_to_face_blender")
        os.makedirs(tmpdir, exist_ok=True)
        base = "blender_lipsync"
        wav_path = os.path.join(tmpdir, base + ".wav")
        json_path = os.path.join(tmpdir, base + ".json")
        # Call Rust CLI
        cmd = [
            BINARY_PATH, "export",
            "--voice", props.voice,
            "--output", wav_path,
            "--text", props.text,
            "--pitch", str(props.pitch),
            "--lipsync", "high",
            "--json-output", json_path
        ]
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            print("[Text-to-Face] CLI output:\n" + result.stdout)
        except Exception as e:
            self.report({'ERROR'}, f"Failed to run text-to-face: {e}")
            return {'CANCELLED'}
        # Import JSON and animate
        if not os.path.exists(json_path):
            self.report({'ERROR'}, f"Lipsync JSON not found: {json_path}")
            return {'CANCELLED'}
        with open(json_path, "r") as f:
            data = json.load(f)
        word_segments = data.get("word_segments", [])
        fps = context.scene.render.fps
        frame_start = int(word_segments[0]["start"] * fps) if word_segments else 1
        frame_end = int(word_segments[-1]["end"] * fps) if word_segments else 1
        clear_viseme_keys(obj, frame_start, frame_end)
        for word in word_segments:
            start = int(word["start"] * fps)
            end = int(word["end"] * fps)
            phonemes = word.get("phonemes", [])
            for ph in phonemes:
                viseme = ARPABET_TO_VISEME.get(ph)
                if viseme:
                    insert_viseme(obj, viseme, start, 1.0)
                    insert_viseme(obj, viseme, end, 0.0)
        self.report({'INFO'}, "Lipsync generated and imported!")
        return {'FINISHED'}

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
        tmp_wav = os.path.join(tempfile.gettempdir(), "text_to_face_preview.wav")
        cmd = [
            BINARY_PATH, "say",
            "--voice", props.voice,
            "--output", tmp_wav,
            "--text", props.text,
            "--pitch", str(props.pitch)
        ]
        try:
            subprocess.run(cmd, check=True)
            bpy.ops.sound.open(filepath=tmp_wav)
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
        layout.operator(TEXTTOFACE_OT_generate_and_import.bl_idname, text="Generate & Import")
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
        items=lambda self, context: [(v, v, "") for v in get_voices()] or [("", "(No voices found)", "")],
    )
    pitch: bpy.props.FloatProperty(
        name="Pitch",
        description="Pitch factor (1.0 = normal)",
        default=1.0,
        min=0.5,
        max=2.0
    )

def register():
    bpy.utils.register_class(TEXTTOFACE_Props)
    bpy.types.Scene.texttoface_props = bpy.props.PointerProperty(type=TEXTTOFACE_Props)
    bpy.utils.register_class(TEXTTOFACE_OT_generate_and_import)
    bpy.utils.register_class(TEXTTOFACE_PT_panel)
    bpy.utils.register_class(TEXTTOFACE_OT_preview_audio)

def unregister():
    bpy.utils.unregister_class(TEXTTOFACE_PT_panel)
    bpy.utils.unregister_class(TEXTTOFACE_OT_generate_and_import)
    del bpy.types.Scene.texttoface_props
    bpy.utils.unregister_class(TEXTTOFACE_Props)
    bpy.utils.unregister_class(TEXTTOFACE_OT_preview_audio) 