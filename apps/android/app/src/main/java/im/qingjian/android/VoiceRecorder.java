// 主动录音，单声道 PCM 16kHz；停止时生成有界 WAV，不落盘录音。
package im.modolanguage.android;
import android.media.AudioFormat;
import android.media.AudioRecord;
import android.media.MediaRecorder;
import java.io.ByteArrayOutputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;

final class VoiceRecorder {
    private volatile boolean stopped;
    void stop() { stopped=true; }
    byte[] record() throws Exception {
        int size=Math.max(4096,AudioRecord.getMinBufferSize(16000,AudioFormat.CHANNEL_IN_MONO,AudioFormat.ENCODING_PCM_16BIT));
        AudioRecord record=new AudioRecord(MediaRecorder.AudioSource.MIC,16000,AudioFormat.CHANNEL_IN_MONO,AudioFormat.ENCODING_PCM_16BIT,size);
        if(record.getState()!=AudioRecord.STATE_INITIALIZED){record.release();throw new IllegalStateException("麦克风无法使用");}
        ByteArrayOutputStream pcm=new ByteArrayOutputStream();byte[] buffer=new byte[size];
        try {
            record.startRecording();long end=System.nanoTime()+30_000_000_000L;
            while(!stopped&&System.nanoTime()<end&&pcm.size()<960000) {
                int n=record.read(buffer,0,Math.min(buffer.length,960000-pcm.size()));
                if(n<0)throw new IllegalStateException("录音读取失败");if(n>0)pcm.write(buffer,0,n);
            }
        } finally { if(record.getRecordingState()==AudioRecord.RECORDSTATE_RECORDING)record.stop();record.release(); }
        byte[] bytes=pcm.toByteArray();if(bytes.length<3200)throw new IllegalStateException("录音太短，请按住至少半秒");
        ByteBuffer wav=ByteBuffer.allocate(44+bytes.length).order(ByteOrder.LITTLE_ENDIAN);
        wav.put("RIFF".getBytes(java.nio.charset.StandardCharsets.US_ASCII)).putInt(36+bytes.length)
            .put("WAVEfmt ".getBytes(java.nio.charset.StandardCharsets.US_ASCII)).putInt(16).putShort((short)1).putShort((short)1)
            .putInt(16000).putInt(32000).putShort((short)2).putShort((short)16)
            .put("data".getBytes(java.nio.charset.StandardCharsets.US_ASCII)).putInt(bytes.length).put(bytes);
        return wav.array();
    }
}
