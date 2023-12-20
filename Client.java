import java.net.Socket;
import java.io.OutputStream;
import java.io.IOException;

public class JavaClient {
    public static void main(String[] args) {
        String hostname = "193.196.38.83";
        int port = 8000;

        Socket socket = null;
        OutputStream os = null;
        try {
            socket = new Socket(hostname, port);
            os = socket.getOutputStream();

            String s = String.format("PX %d %d %02x%02x%02x\n", 0, 0, 255, 255, 255);
            os.write(s.getBytes());
            // TODO add custom functionality

            socket.close();
        } catch (IOException e) {
            e.printStackTrace(System.out);
        }
    }
}
