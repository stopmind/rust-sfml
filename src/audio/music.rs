use {
    super::{sound_channel::SoundChannel, sound_source::SoundSource},
    crate::{
        IntoSfResult, SfResult,
        audio::TimeSpan,
        cpp::CppVector,
        ffi,
        system::{InputStream, Time, Vector3f},
    },
    std::{
        ffi::{CString, c_uint, c_void},
        io::Empty
    },
};

/// Streamed music played from an audio file.
///
/// `Music`s are sounds that are streamed rather than completely loaded in memory.
///
/// This is especially useful for compressed musics that usually take hundreds of MB when they are
/// uncompressed: by streaming it instead of loading it entirely, you avoid saturating the memory
/// and have almost no loading delay. This implies that the underlying resource
/// (file, stream or memory buffer) must remain valid for the lifetime of the `Music` object.
///
/// Apart from that, a `Music` has almost the same features as the
/// [`SoundBuffer`] / [`Sound`] pair: you can play/pause/stop it, request its parameters
/// (channels, sample rate), change the way it is played (pitch, volume, 3D position, ...), etc.
///
/// As a sound stream, a music is played in its own thread in order not to block the rest of the
/// program. This means that you can leave the music alone after calling [`play`],
/// it will manage itself very well.
///
/// [`play`]: Music::play
/// [`SoundBuffer`]: crate::audio::SoundBuffer
/// [`Sound`]: crate::audio::Sound
#[derive(Debug)]
pub struct Music<'src, S = Empty> {
    music: *mut ffi::audio::sfMusic,
    stream: Option<InputStream<'src, S>>,
}

// SAFETY: An `sfMusic` isn't tied to a particular thread, so it can be sent between threads safely.
unsafe impl Send for Music<'_> {}

// SAFETY: An `&Music` only allows access to methods which read the status of the music, which is
// fine to do from multiple threads at once. Thus it is safe to pass `&Music` between threads.
unsafe impl Sync for Music<'_> {}

/// Creating and opening
impl<'src, S> Music<'src, S> {
    /// Create a new (empty) `Music`.
    pub fn new() -> SfResult<Self> {
        Ok(Self {
            music: unsafe { ffi::audio::sfMusic_new() },
            stream: None,
        })
    }

    /// Create a new `Music` by "opening" it from a stream
    ///
    /// See [`Self::open_from_stream`].
    pub fn from_stream(stream: &'src mut InputStream<'src, S>) -> SfResult<Self> {
        let mut new = Self::new()?;
        new.open_from_stream(stream)?;
        Ok(new)
    }
    /// Create a new `Music` by "opening" it from a stream, with ownership.
    ///
    /// See [`Self::open_from_stream`].
    pub fn from_stream_owned(stream: InputStream<'src, S>) -> SfResult<Self> {
        let mut new = Self::new()?;
        new.open_from_stream_owned(stream)?;
        Ok(new)
    }

    /// Open music from a stream (a struct implementing Read and Seek)
    ///
    /// This function doesn't start playing the music (call [`play`] to do so).
    /// Here is a complete list of all the supported audio formats:
    /// ogg, wav, flac, aiff, au, raw, paf, svx, nist, voc, ircam,
    /// w64, mat4, mat5 pvf, htk, sds, avr, sd2, caf, wve, mpc2k, rf64.
    ///
    /// # Arguments
    /// * stream - Your struct, implementing Read and Seek
    ///
    /// [`play`]: Music::play
    pub fn open_from_stream(
        &mut self,
        stream: &'src mut InputStream<'src, S>,
    ) -> SfResult<()> {
        unsafe { ffi::audio::sfMusic_openFromStream(self.music, &raw mut *stream.stream) }
            .into_sf_result()
    }

    /// Open music from a stream (a struct implementing Read and Seek), with ownership.
    ///
    /// This function doesn't start playing the music (call [`play`] to do so).
    /// Here is a complete list of all the supported audio formats:
    /// ogg, wav, flac, aiff, au, raw, paf, svx, nist, voc, ircam,
    /// w64, mat4, mat5 pvf, htk, sds, avr, sd2, caf, wve, mpc2k, rf64.
    ///
    /// # Arguments
    /// * stream - Your struct, implementing Read and Seek
    ///
    /// [`play`]: Music::play
    pub fn open_from_stream_owned(
        &mut self,
        stream: InputStream<'src, S>,
    ) -> SfResult<()> {
        unsafe { ffi::audio::sfMusic_openFromStream(self.music, stream.stream.0.as_ptr()) }
            .into_sf_result()
            .map(|_| {
                self.stream = Some(stream);
                ()
            })

    }
}

/// Creating and opening from file and memory
impl<'src> Music<'src> {
    /// Create a new `Music` by opening a music file
    ///
    /// See [`Self::open_from_file`].
    pub fn from_file(filename: &str) -> SfResult<Self> {
        let mut new = Self::new()?;
        new.open_from_file(filename)?;
        Ok(new)
    }
    /// Create a new `Music` by "opening" it from music data in memory
    ///
    /// See [`Self::open_from_memory`].
    pub fn from_memory(data: &'src [u8]) -> SfResult<Self> {
        let mut new = Self::new()?;
        new.open_from_memory(data)?;
        Ok(new)
    }
    /// Open a new file for playback
    ///
    /// This function doesn't start playing the music (call [`play`] to do so).
    /// Here is a complete list of all the supported audio formats:
    /// ogg, wav, flac, aiff, au, raw, paf, svx, nist, voc, ircam,
    /// w64, mat4, mat5 pvf, htk, sds, avr, sd2, caf, wve, mpc2k, rf64.
    ///
    /// # Arguments
    /// * filename - Path of the music file to open
    ///
    /// [`play`]: Music::play
    pub fn open_from_file(&mut self, filename: &str) -> SfResult<()> {
        let c_str = CString::new(filename)?;
        unsafe { ffi::audio::sfMusic_openFromFile(self.music, c_str.as_ptr()) }.into_sf_result()
    }

    /// Create a new music and open it from memory
    ///
    /// This function doesn't start playing the music (call [`play`] to do so).
    /// Here is a complete list of all the supported audio formats:
    /// ogg, wav, flac, aiff, au, raw, paf, svx, nist, voc, ircam,
    /// w64, mat4, mat5 pvf, htk, sds, avr, sd2, caf, wve, mpc2k, rf64.
    ///
    /// # Arguments
    /// * `data` - Slice of music data in memory
    ///
    /// [`play`]: Music::play
    pub fn open_from_memory(&mut self, data: &'src [u8]) -> SfResult<()> {
        unsafe { ffi::audio::sfMusic_openFromMemory(self.music, data.as_ptr(), data.len()) }
            .into_sf_result()
    }
}

/// Playback
impl<S> Music<'_, S> {
    /// Start or resume playing a music
    ///
    /// This function starts the music if it was stopped, resumes
    /// it if it was paused, and restarts it from beginning if it
    /// was it already playing.
    /// This function uses its own thread so that it doesn't block
    /// the rest of the program while the music is played.
    pub fn play(&mut self) {
        unsafe { ffi::audio::sfMusic_play(self.music) }
    }

    /// Pause a music
    ///
    /// This function pauses the music if it was playing,
    /// otherwise (music already paused or stopped) it has no effect.
    pub fn pause(&mut self) {
        unsafe { ffi::audio::sfMusic_pause(self.music) }
    }

    /// Stop playing a music
    ///
    /// This function stops the music if it was playing or paused,
    /// and does nothing if it was already stopped.
    /// It also resets the playing position (unlike pause).
    pub fn stop(&mut self) {
        unsafe { ffi::audio::sfMusic_stop(self.music) }
    }
}

/// Query properties
impl<S> Music<'_, S> {
    /// Tell whether or not a music is in loop mode
    ///
    /// Return true if the music is looping, false otherwise
    #[must_use]
    pub fn is_looping(&self) -> bool {
        unsafe { ffi::audio::sfMusic_isLooping(self.music) }
    }
    /// Get the total duration of a music
    ///
    /// Return Music duration
    #[must_use]
    pub fn duration(&self) -> Time {
        unsafe { Time::from_raw(ffi::audio::sfMusic_getDuration(self.music)) }
    }
    /// Return the number of channels of a music
    ///
    /// 1 channel means a mono sound, 2 means stereo, etc.
    ///
    /// Return the number of channels
    #[must_use]
    pub fn channel_count(&self) -> u32 {
        unsafe { ffi::audio::sfMusic_getChannelCount(self.music) }
    }
    /// Get the sample rate of a music
    ///
    /// The sample rate is the number of audio samples played per
    /// second. The higher, the better the quality.
    ///
    /// Return the sample rate, in number of samples per second
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        unsafe { ffi::audio::sfMusic_getSampleRate(self.music) }
    }

    /// Get the map of position in sample frame to sound channel
    ///
    /// This is used to map a sample in the sample stream to a
    /// position during spatialization.
    ///
    /// Return Map of position in sample frame to sound channel
    #[must_use]
    pub fn channel_map(&self) -> &'static CppVector<SoundChannel> {
        unsafe { &*ffi::audio::sfMusic_getChannelMap(self.music) }
    }

    /// Get the current playing position of a music
    ///
    /// Return the current playing position
    #[must_use]
    pub fn playing_offset(&self) -> Time {
        unsafe { Time::from_raw(ffi::audio::sfMusic_getPlayingOffset(self.music)) }
    }

    /// Get the positions of the of the music's looping sequence.
    ///
    /// # Warning
    /// Since [`set_loop_points`] performs some adjustments on the provided values and
    /// rounds them to internal samples, a call to [`loop_points`] is not guaranteed to
    /// return the same times passed into a previous call to [`set_loop_points`].
    /// However, it is guaranteed to return times that will map to the
    /// valid internal samples of this [`Music`] if they are later passed to [`set_loop_points`].
    ///
    /// [`set_loop_points`]: Music::set_loop_points
    /// [`loop_points`]: Music::loop_points
    #[must_use]
    pub fn loop_points(&self) -> TimeSpan {
        TimeSpan::from_raw(unsafe { ffi::audio::sfMusic_getLoopPoints(self.music) })
    }
}

/// Set properties
impl<S> Music<'_, S> {
    /// Sets whether this music should loop or not.
    ///
    /// If `true`, the music will restart from beginning after
    /// reaching the end and so on, until it is stopped or
    /// `set_looping(false)` is called.
    ///
    /// By default, the music will *not* loop.
    pub fn set_looping(&mut self, looping: bool) {
        unsafe { ffi::audio::sfMusic_setLooping(self.music, looping) }
    }
    /// Change the current playing position of a music
    ///
    /// The playing position can be changed when the music is
    /// either paused or playing.
    ///
    /// # Arguments
    /// * timeOffset - New playing position
    pub fn set_playing_offset(&mut self, time_offset: Time) {
        unsafe { ffi::audio::sfMusic_setPlayingOffset(self.music, time_offset.raw()) }
    }
    /// Sets the beginning and end of the music's looping sequence.
    ///
    /// Loop points allow one to specify a pair of positions such that, when the music is
    /// enabled for looping, it will seamlessly seek to the beginning whenever it encounters
    /// the end. Valid ranges for timePoints.offset and timePoints.length are
    /// `[0, Dur)` and `(0, Dur-offset]` respectively, where `Dur` is the value returned by
    /// `duration`. Note that the EOF "loop point" from the end to the beginning of the
    /// stream is still honored, in case the caller seeks to a point after the end of the
    /// loop range. This function can be safely called at any point after a stream is opened,
    /// and will be applied to a playing sound without affecting the current playing offset.
    pub fn set_loop_points(&mut self, time_points: TimeSpan) {
        unsafe { ffi::audio::sfMusic_setLoopPoints(self.music, time_points.into_raw()) }
    }
}

impl<S> SoundSource for Music<'_, S> {
    fn set_pitch(&mut self, pitch: f32) {
        unsafe { ffi::audio::sfMusic_setPitch(self.music, pitch) }
    }
    fn set_volume(&mut self, volume: f32) {
        unsafe { ffi::audio::sfMusic_setVolume(self.music, volume) }
    }
    fn set_position<P: Into<Vector3f>>(&mut self, position: P) {
        unsafe { ffi::audio::sfMusic_setPosition(self.music, position.into()) }
    }
    fn set_relative_to_listener(&mut self, relative: bool) {
        unsafe { ffi::audio::sfMusic_setRelativeToListener(self.music, relative) }
    }
    fn set_min_distance(&mut self, distance: f32) {
        unsafe { ffi::audio::sfMusic_setMinDistance(self.music, distance) }
    }
    fn set_attenuation(&mut self, attenuation: f32) {
        unsafe { ffi::audio::sfMusic_setAttenuation(self.music, attenuation) }
    }
    fn pitch(&self) -> f32 {
        unsafe { ffi::audio::sfMusic_getPitch(self.music) }
    }
    fn volume(&self) -> f32 {
        unsafe { ffi::audio::sfMusic_getVolume(self.music) }
    }
    fn position(&self) -> Vector3f {
        unsafe { ffi::audio::sfMusic_getPosition(self.music) }
    }
    fn is_relative_to_listener(&self) -> bool {
        unsafe { ffi::audio::sfMusic_isRelativeToListener(self.music) }
    }
    fn min_distance(&self) -> f32 {
        unsafe { ffi::audio::sfMusic_getMinDistance(self.music) }
    }
    fn attenuation(&self) -> f32 {
        unsafe { ffi::audio::sfMusic_getAttenuation(self.music) }
    }
    fn set_pan(&mut self, pan: f32) {
        unsafe { ffi::audio::sfMusic_setPan(self.music, pan) }
    }
    fn set_spatialization_enabled(&mut self, enabled: bool) {
        unsafe { ffi::audio::sfMusic_setSpatializationEnabled(self.music, enabled) }
    }
    fn set_direction<P: Into<Vector3f>>(&mut self, direction: P) {
        unsafe { ffi::audio::sfMusic_setDirection(self.music, direction.into()) }
    }
    fn set_cone(&mut self, cone: super::sound_source::Cone) {
        unsafe { ffi::audio::sfMusic_setCone(self.music, cone.into()) }
    }
    fn set_velocity<P: Into<Vector3f>>(&mut self, velocity: P) {
        unsafe { ffi::audio::sfMusic_setVelocity(self.music, velocity.into()) }
    }
    fn set_doppler_factor(&mut self, factor: f32) {
        unsafe { ffi::audio::sfMusic_setDopplerFactor(self.music, factor) }
    }
    fn set_directional_attenuation_factor(&mut self, factor: f32) {
        unsafe { ffi::audio::sfMusic_setDirectionalAttenuationFactor(self.music, factor) }
    }
    fn set_max_distance(&mut self, distance: f32) {
        unsafe { ffi::audio::sfMusic_setMaxDistance(self.music, distance) }
    }
    fn set_min_gain(&mut self, gain: f32) {
        unsafe { ffi::audio::sfMusic_setMinGain(self.music, gain) }
    }
    fn set_max_gain(&mut self, gain: f32) {
        unsafe { ffi::audio::sfMusic_setMaxGain(self.music, gain) }
    }
    fn set_effect_processor(&mut self, effect_processor: super::sound_source::EffectProcessor) {
        let (cb, user_data) = match effect_processor {
            Some(cb) => {
                let boxed = Box::new(cb);
                let trampoline: unsafe extern "C" fn(
                    *const f32,
                    *mut c_uint,
                    *mut f32,
                    *mut c_uint,
                    c_uint,
                    *mut c_void,
                ) = ffi::audio::effect_processor_trampoline;
                (Some(trampoline), Box::into_raw(boxed).cast::<c_void>())
            }
            None => (None, std::ptr::null_mut()),
        };

        unsafe {
            ffi::audio::sfMusic_setEffectProcessor(self.music, cb, user_data);
        }
    }
    fn pan(&self) -> f32 {
        unsafe { ffi::audio::sfMusic_getPan(self.music) }
    }
    fn is_spatialization_enabled(&self) -> bool {
        unsafe { ffi::audio::sfMusic_isSpatializationEnabled(self.music) }
    }
    fn direction(&self) -> Vector3f {
        unsafe { ffi::audio::sfMusic_getDirection(self.music) }
    }
    fn cone(&self) -> super::sound_source::Cone {
        unsafe { ffi::audio::sfMusic_getCone(self.music).into() }
    }
    fn velocity(&self) -> Vector3f {
        unsafe { ffi::audio::sfMusic_getVelocity(self.music) }
    }
    fn doppler_factor(&self) -> f32 {
        unsafe { ffi::audio::sfMusic_getDopplerFactor(self.music) }
    }
    fn directional_attenuation_factor(&self) -> f32 {
        unsafe { ffi::audio::sfMusic_getDirectionalAttenuationFactor(self.music) }
    }
    fn get_max_distance(&self) -> f32 {
        unsafe { ffi::audio::sfMusic_getMaxDistance(self.music) }
    }
    fn get_min_gain(&self) -> f32 {
        unsafe { ffi::audio::sfMusic_getMinGain(self.music) }
    }
    fn get_max_gain(&self) -> f32 {
        unsafe { ffi::audio::sfMusic_getMaxGain(self.music) }
    }
    fn status(&self) -> super::sound_source::Status {
        unsafe { ffi::audio::sfMusic_getStatus(self.music) }
    }
}

impl<S> Drop for Music<'_, S> {
    fn drop(&mut self) {
        unsafe {
            ffi::audio::sfMusic_del(self.music);
        }
    }
}
