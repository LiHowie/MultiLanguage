// 将随包产品数据复制到应用私有目录，以供 Rust mmap 加载。
package im.modolanguage.android;

import android.content.Context;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.IOException;

final class ProductData {
    static File prepare(Context context) throws IOException {
        File data = new File(context.getFilesDir(), "data-v3");
        if (!data.isDirectory() && !data.mkdirs()) throw new IOException("无法创建词库目录");
        String[] files = {"dict.qj", "lm.qj", "glossary-en.qj", "glossary-ja.qj", "glossary-es.qj"};
        for (String name : files) {
            File target = new File(data, name);
            if (target.isFile() && target.length() > 0) continue;
            File temporary = new File(data, name + ".partial");
            try (InputStream input = context.getAssets().open("data/" + name);
                 FileOutputStream output = new FileOutputStream(temporary)) {
                byte[] bytes = new byte[65536];
                int count;
                while ((count = input.read(bytes)) >= 0) output.write(bytes, 0, count);
                output.getFD().sync();
            }
            if (!temporary.renameTo(target)) throw new IOException("词库准备失败：" + name);
        }
        return data;
    }
    private ProductData() { }
}
