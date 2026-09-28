from PIL.Image import open, Image

im: Image = open("assets/box_bg_green.png")

for x in range(im.width):
    for y in range(im.height):
        (r,g,b,a) = im.getpixel((x,y))
        if a and g > r * 1.5 and g > b:
            im.putpixel((x,y), (int(g * 1.1), int(r * .9), int(b * .9), a))

im.save("generated_assets/box_bg_red.png")