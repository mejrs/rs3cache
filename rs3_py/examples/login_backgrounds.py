import os, enum
from PIL import Image
from rs3 import Sprites, get_struct_configs, get_enum_configs

OUTPUT_FOLDER = "login_backgrounds"

print(f"Saving login backgrounds to {os.path.abspath(OUTPUT_FOLDER)}")

try:
    os.mkdir(OUTPUT_FOLDER)
except FileExistsError:
    pass

class Param(enum.IntEnum):
    Slideshow_interval = 4234
    Slideshow = 6589

class Segments(enum.Enum):
        TOP_LEFT = (6533, (0, 0))
        TOP_MIDDLE = (6534, (640, 0))
        TOP_RIGHT = (6535, (1280, 0))
        LEFT = (6536, (0, 360))
        MIDDLE = (6537, (640, 360))
        RIGHT = (6538, (1280, 360))
        BOTTOM_LEFT = (6539, (0, 720))
        BOTTOM_MIDDLE = (6540, (640, 720))
        BOTTOM_RIGHT = (6541, (1280, 720))

def process(struct, name):
    canvas = Image.new("RGBA", (1920, 1080))
    for segment in Segments:
        param, offset = segment.value
        image_id = struct.params[param]
        segment = sprites[image_id][0].image()
        canvas.paste(segment, offset)
    canvas.save(f"{OUTPUT_FOLDER}/{name}.png")

sprites = Sprites()

enums = get_enum_configs()
structs = get_struct_configs()
background_names = enums[12590].variants
backgrounds = enums[12591].variants

for id, key in backgrounds.items():
    struct = structs[key]
    # Get and sanitize the filename
    name = background_names.get(id, f"-Default- {id}").replace("/", "").replace(":", "")

    # Some point to a slideshow, which we process individually
    if slideshow_key := struct.params.get(Param.Slideshow):
        slideshow_backgrounds = enums[slideshow_key].variants
        for i, slideshow_key in enumerate(slideshow_backgrounds.values()):
            process(structs[slideshow_key], f"{name}-{i}")

    # do nothing
    elif struct.params.get(Param.Slideshow_interval):
        pass
    else:
        process(struct, name)
