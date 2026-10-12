//! Windows 麦克风 PCM 录音，不保存原始录音到磁盘。
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use windows::Win32::Media::Audio::{
    CALLBACK_NULL, HWAVEIN, WAVE_MAPPER, WAVEFORMATEX, WAVEHDR, waveInAddBuffer, waveInClose,
    waveInOpen, waveInPrepareHeader, waveInReset, waveInStart, waveInUnprepareHeader,
};
use windows_reactor::CancellationToken;

pub(super) fn record(
    stop: &AtomicBool,
    discard: &AtomicBool,
    cancel: &CancellationToken,
) -> Result<Vec<u8>, String> {
    let mut pcm = vec![0u8; 960000];
    let format = WAVEFORMATEX {
        wFormatTag: 1,
        nChannels: 1,
        nSamplesPerSec: 16000,
        nAvgBytesPerSec: 32000,
        nBlockAlign: 2,
        wBitsPerSample: 16,
        cbSize: 0,
    };
    let mut device = HWAVEIN::default();
    let size = std::mem::size_of::<WAVEHDR>() as u32;
    let mut header = Box::new(WAVEHDR {
        lpData: windows::core::PSTR(pcm.as_mut_ptr()),
        dwBufferLength: pcm.len() as u32,
        ..Default::default()
    });
    // PCM 和头在 reset/unprepare 返回前保持固定地址，驱动只在录音阶段写入。
    unsafe {
        if waveInOpen(
            Some(&mut device),
            WAVE_MAPPER,
            &format,
            None,
            None,
            CALLBACK_NULL,
        ) != 0
        {
            return Err("麦克风无法使用，请在 Windows 隐私设置中允许录音".to_owned());
        }
        if waveInPrepareHeader(device, &mut *header, size) != 0 {
            waveInClose(device);
            return Err("录音初始化失败".to_owned());
        }
        let initialized =
            waveInAddBuffer(device, &mut *header, size) == 0 && waveInStart(device) == 0;
        if initialized {
            let started = Instant::now();
            while !stop.load(Ordering::Relaxed)
                && !discard.load(Ordering::Relaxed)
                && !cancel.is_cancelled()
                && started.elapsed() < Duration::from_secs(30)
            {
                std::thread::sleep(Duration::from_millis(30));
            }
        }
        waveInReset(device);
        waveInUnprepareHeader(device, &mut *header, size);
        waveInClose(device);
        if !initialized {
            return Err("麦克风录音失败".to_owned());
        }
    }
    if cancel.is_cancelled() || discard.load(Ordering::Relaxed) {
        return Err("录音已取消".to_owned());
    }
    let length = (header.dwBytesRecorded as usize).min(pcm.len()) / 2 * 2;
    if length < 3200 {
        return Err("录音太短，请至少录制半秒".to_owned());
    }
    pcm.truncate(length);
    let mut wav = Vec::with_capacity(44 + length);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + length as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&16000u32.to_le_bytes());
    wav.extend_from_slice(&32000u32.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(length as u32).to_le_bytes());
    wav.extend_from_slice(&pcm);
    Ok(wav)
}
