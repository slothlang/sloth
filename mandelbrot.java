public class mandelbrot {
    public static void main(String[] args) {
                        // char escCode = 0x1B;
                        // System.out.printf("%c[%d;%dH", escCode, 20, 10);
                        // System.out.print("█");
                        // System.out.printf("%c[%d;%dH", escCode, 20, 20);
                        // System.out.print("█");
                        // System.out.printf("%c[%d;%dH", escCode, 30, 10);
                        // System.out.print("█");

        double size = 800;
        double maxVal = 4;
        double maxIter = 50;
        double plane = 4;
        int x = 0;
        while (x < size) {
            int y = 0;
            while (y < size) {
                double cReal = (x * plane / size) - 2.0;
                double cImg = (y * plane / size) - 2.0;
                double zReal = 0.0;
                double zImg = 0.0;
                double count = 0.0;
                while ((zReal * zReal + zImg * zImg) <= maxVal && count < maxIter) {
                    var temp = (zReal * zReal) - (zImg * zImg) + cReal;
                    zImg = 2.0 * zReal * zImg + cImg;
                    zReal = temp;
                    count = count + 1.0;
                    if (count == maxIter) {
                        char escCode = 0x1B;
                        System.out.printf("%c[%d;%dH", escCode, x, y);
                        System.out.print("█");
                    }
                }
                y = y + 1;
            }
            x = x + 1;
        }
    }
}
