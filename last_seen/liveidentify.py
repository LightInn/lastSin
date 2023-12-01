from PIL import ImageGrab
import cv2
import numpy as np
import Processing

# Shapes of the League of Legends Minimap, adjust to fit your screen
width = 275
height = 275
"""
define the box to crop the minimap
        (x1,y1)
             ________________________
            |                        |
            |                        |
            |                        |
            |                        |
            |        Minimap         |
            |                        |
            |                        |
            |                        |
            |                        |
            |________________________|
                                   (x2,y2)
"""
x2 = 2560
y2 = 1440
x1 = x2 - width
y1 = y2 - height

fps = 240


def getimg():
    image = ImageGrab.grab((x1, y1, x2, y2))
    image_array = np.array(image)
    image_bgr = cv2.cvtColor(image_array, cv2.COLOR_RGB2BGR)
    # processed = photo2vid.process(image_bgr)
    # resized = cv2.resize(processed, (width,height))
    return image_bgr


def main():
    while True:
        img = getimg()
        img = Processing.process(img)
        cv2.imshow("", img)
        cv2.waitKey(int(1000 / fps))


if __name__ == '__main__':
    main()
