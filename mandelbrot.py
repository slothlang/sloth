size = 200

# constants:
MaxValue = 4
MaxIterations = 50
planeWidth = 4

for x in range(0, size):
    for y in range(0, size): # for each pixel do:
        cReal = (x * planeWidth / size) - 2
        cImg =  (y * planeWidth / size) - 2
        zReal = 0
        zImg = 0
        count = 0
        while (zReal*zReal + zImg*zImg) <= MaxValue and count < MaxIterations:
            temp = (zReal * zReal) - (zImg * zImg) + cReal
            zImg = 2 * zReal * zImg + cImg
            zReal = temp
            count += 1

        if count == MaxIterations:
            print(f"\x1b[{x};{y}H*")
