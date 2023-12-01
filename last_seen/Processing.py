from math import sqrt
import cv2

data = []
distance_threshold = 20
radius = 11

class Enemy:
    last_seen = 0
    existence = 0
    coord = (0, 0)
    pic = None

def new_enemy(coords, cropped):
    print("new enemy")
    enemy = Enemy()
    enemy.coord = coords
    enemy.pic = cropped
    data.append(enemy)

def refresh(enemy: Enemy, coord: tuple):
    enemy.coord = coord
    enemy.last_seen = 0
    enemy.existence += 1

def nearest(coord):
    nearest = None
    if len(data) > 0:
        nearest = min(data,
                      key=lambda x: distance(x.coord, (coord[0], coord[1])))
        if distance(nearest.coord, (coord[0], coord[1])) > distance_threshold:
            nearest = None
    return nearest


def distance(co1, co2):
    return sqrt(pow(abs(co1[0] - co2[0]), 2) + pow(abs(co1[1] - co2[1]), 2))


def compare_hist(img1, img2):
    h1 = cv2.calcHist([img1], [0], None, [256], [0, 256])
    h2 = cv2.calcHist([img2], [0], None, [256], [0, 256])
    compare = cv2.compareHist(h1, h2, 0)
    print("compare: " + str(compare))

    return compare


def process(image):
    """
    An image of minimap ---> show last seen position of champions
    """

    coords = []

    b, g, r = cv2.split(image)
    in_range_r = cv2.inRange(r, 120, 255)
    in_range_g = cv2.inRange(g, 120, 255)
    in_range_b = cv2.inRange(b, 120, 255)
    induction = in_range_r - in_range_g - in_range_b

    # regarder la map et detecter les ennemis
    circles = cv2.HoughCircles(induction, cv2.HOUGH_GRADIENT, 1, 10, param1=30, param2=15, minRadius=9, maxRadius=30)
    if circles is not None:
        for n in range(circles.shape[1]):

            coord = (int(circles[0][n][0]), int(circles[0][n][1]))

            near = nearest(coord)
            if near is not None and near.last_seen < 5:
                refresh(near, coord)
            else:
                cropped = cv2.resize(
                    image[coord[1] - radius:coord[1] + radius, coord[0] - radius:coord[0] + radius].copy(), (24, 24))
                if len(data) < 5:
                    new_enemy(coord, cropped)
                else:
                    similar = max(data, key=lambda x: compare_hist(x.pic, cropped))
                    print("best match : ", compare_hist(similar.pic, cropped))
                    refresh(similar, coord)

    # si un ennemi n'est pas detecte depuis longtemps
    for n in filter(lambda x: x.last_seen > 5, data):
        # si sont existance est inferieur a 4, le supprimer de la liste
        if n.existence < 4:
            data.remove(n)
        else:
            # si last_seen est superieur a 666, ne plus l'afficher
            if n.last_seen > 666:
                pass
            else:
                # sinon, overlay n.pic sur l'image

                pic_bw_resized = cv2.resize(cv2.cvtColor(n.pic, cv2.COLOR_BGR2GRAY), (radius, radius))
                image[n.coord[1] + 5 - radius:n.coord[1] + 5, n.coord[0] + 5 - radius:n.coord[0] + 5] = cv2.cvtColor(
                    pic_bw_resized, cv2.COLOR_GRAY2BGR)

                # pic_resized = cv2.resize(n.pic, (2 * radius, 2 * radius))
                # image[n.coord[1] - radius:n.coord[1] + radius, n.coord[0] - radius:n.coord[0] + radius] = pic_resized

                # cv2.rectangle(image, (n.coord[0] - radius, n.coord[1] - radius),
                #               (n.coord[0] + radius, n.coord[1] + radius), (0, 0, 255), 1)

    # augmenter last_seen de 1 pour chaque ennemi dans la liste
    for n in data:
        n.last_seen += 1

    return image
