size = 800.0
maxVal = 4.0
maxIter = 50.0
plane = 4.0
x = 0;
while x < size:
    y = 0;
    while y < size:
        cReal = (x * plane / size) - 2.0
        cImg = (y * plane / size) - 2.0
        zReal = 0.0
        zImg = 0.0
        count = 0.0
        while (zReal * zReal + zImg * zImg) <= maxVal and count < maxIter:
            temp = (zReal * zReal) - (zImg * zImg) + cReal
            zImg = 2.0 * zReal * zImg + cImg
            zReal = temp
            count = count + 1.0
            if count == maxIter:
                print(f"\x1b[{x};{y}H", end="")
                print("█", end="")
        y = y + 1
    x = x + 1
