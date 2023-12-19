import java.net.Socket;
import java.io.OutputStream;
import java.io.IOException;

public class Client {
    public static void main(String[] args) {
        String hostname = "127.0.0.1";
        int port = 1337;

        Socket socket = null;
        OutputStream os = null;
        try {
            socket = new Socket(hostname, port);
            os = socket.getOutputStream();
            for (int i = 0; i < 5; i++) {
                for (int j = 0; j < 5; j++) {
                    String s = String.format("PX %d %d %02x%02x%02x\n", i, j, 255, 255, 0);
                    System.out.print(s);
                    os.write(s.getBytes());
                }
            }
            socket.close();
        } catch (IOException e) {
            e.printStackTrace(System.out);
        }
    }
}
