#[cfg(creusot)]
use creusot_std::prelude::inv;
#[allow(unused_imports)]
use creusot_std::prelude::{
    DeepModel, Int, Invariant, Seq, View, ensures, invariant, logic, pearlite, requires, trusted,
};

#[cfg_attr(not(creusot), derive(Debug))]
#[cfg_attr(not(creusot), derive(thiserror::Error))]
#[cfg_attr(
    all(feature = "serde", not(creusot)),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(all(feature = "defmt", not(creusot)), derive(defmt::Format))]
#[cfg_attr(not(creusot), error("out of bounds error during encoding"))]
pub struct DestBufTooSmallError;

impl View for DestBufTooSmallError {
    type ViewTy = ();

    #[logic(open)]
    fn view(self) -> Self::ViewTy {}
}

impl DeepModel for DestBufTooSmallError {
    type DeepModelTy = ();

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {}
}

impl Invariant for DestBufTooSmallError {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { true }
    }
}

impl PartialEq for DestBufTooSmallError {
    #[trusted]
    #[ensures(result)]
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for DestBufTooSmallError {}

/// The [`EncoderState`] is used to track the current state of a streaming encoder. This struct
/// does not contain the output buffer (or a reference to one), and can be used when streaming the
/// encoded output to a custom data type
///
/// **IMPORTANT NOTE**: When implementing a custom streaming encoder,
/// the [`EncoderState`] state machine assumes that the output buffer
/// **ALREADY** contains a single placeholder byte, and no other bytes.
/// This placeholder byte will be later modified with the first distance
/// to the next header/zero byte.
#[derive(Clone)]
#[cfg_attr(not(creusot), derive(Debug))]
pub struct EncoderState {
    code_idx: usize,
    num_bt_sent: u8,
    offset_idx: u8,
}

impl View for EncoderState {
    type ViewTy = (usize, u8, u8);

    #[logic]
    fn view(self) -> Self::ViewTy {
        (self.code_idx, self.num_bt_sent, self.offset_idx)
    }
}

impl Invariant for EncoderState {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! {
            1 <= self@.1@ && self@.1@ <= 254 && self@.2 == self@.1
        }
    }
}

/// [`PushResult`] is used to represent the changes to an (encoded)
/// output data buffer when an unencoded byte is pushed into [`EncoderState`].
pub enum PushResult {
    /// The returned byte should be placed at the current end of the data buffer
    AddSingle(u8),

    /// The byte at the given index should be replaced with the given byte.
    /// Additionally, a placeholder byte should be inserted at the current
    /// end of the output buffer to be later modified
    ModifyFromStartAndSkip((usize, u8)),

    /// The byte at the given index should be replaced with the given byte.
    /// Then, the last u8 in this tuple should be inserted at the end of the
    /// current output buffer. Finally, a placeholder byte should be inserted at
    /// the current end of the output buffer to be later modified if the encoding process is
    /// not done yet.
    ModifyFromStartAndPushAndSkip((usize, u8, u8)),
}

impl View for PushResult {
    type ViewTy = (u8, usize, u8, u8);

    #[logic]
    fn view(self) -> Self::ViewTy {
        match self {
            PushResult::AddSingle(byte) => (0u8, 0usize, 0u8, byte),
            PushResult::ModifyFromStartAndSkip((index, code)) => (1u8, index, code, 0u8),
            PushResult::ModifyFromStartAndPushAndSkip((index, code, byte)) => {
                (2u8, index, code, byte)
            }
        }
    }
}

impl Invariant for PushResult {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { true }
    }
}

impl Default for EncoderState {
    /// Create a default initial state representation for a COBS encoder
    #[ensures(result@ == (0usize, 1u8, 1u8))]
    fn default() -> Self {
        Self {
            code_idx: 0,
            num_bt_sent: 1,
            offset_idx: 1,
        }
    }
}

impl EncoderState {
    /// Push a single unencoded byte into the encoder state machine
    #[requires(inv(*self))]
    #[requires((data == 0u8 || (*self)@.1 == 254u8) ==>
        (*self)@.0@ + (*self)@.2@ + 1 <= usize::MAX@)]
    #[ensures(inv(^self))]
    #[ensures(match result {
        PushResult::AddSingle(byte) => data != 0u8 && (*self)@.1 < 254u8 && byte == data,
        PushResult::ModifyFromStartAndSkip((index, code)) =>
            data == 0u8 && index == (*self)@.0 && code == (*self)@.1,
        PushResult::ModifyFromStartAndPushAndSkip((index, code, byte)) =>
            data != 0u8 && (*self)@.1 == 254u8 && index == (*self)@.0
                && code == 255u8 && byte == data,
    })]
    pub fn push(&mut self, data: u8) -> PushResult {
        if data == 0 {
            let ret = PushResult::ModifyFromStartAndSkip((self.code_idx, self.num_bt_sent));
            self.code_idx += usize::from(self.offset_idx);
            self.num_bt_sent = 1;
            self.offset_idx = 1;
            ret
        } else {
            self.num_bt_sent += 1;
            self.offset_idx += 1;

            if 0xFF == self.num_bt_sent {
                let ret = PushResult::ModifyFromStartAndPushAndSkip((
                    self.code_idx,
                    self.num_bt_sent,
                    data,
                ));
                self.num_bt_sent = 1;
                self.code_idx += usize::from(self.offset_idx);
                self.offset_idx = 1;
                ret
            } else {
                PushResult::AddSingle(data)
            }
        }
    }

    /// Finalize the encoding process for a single message.
    /// The byte at the given index should be replaced with the given value,
    /// and the sentinel value (typically 0u8) must be inserted at the current
    /// end of the output buffer, serving as a framing byte.
    #[ensures(result == (self@.0, self@.1))]
    pub fn finalize(self) -> (usize, u8) {
        (self.code_idx, self.num_bt_sent)
    }
}

/// The [`CobsEncoder`] type is used to encode a stream of bytes to a given mutable output slice.
///
/// This is often useful when heap data structures are not available, or when not all message bytes
/// are received at a single point in time.
#[cfg_attr(not(creusot), derive(Debug))]
pub struct CobsEncoder<'a> {
    dest: &'a mut [u8],
    dest_idx: usize,
    state: EncoderState,
    might_be_done: bool,
}

impl View for CobsEncoder<'_> {
    type ViewTy = (Int, Int, Int, Int, bool);

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { (
            self.dest@.len(),
            self.dest_idx@,
            self.state.code_idx@,
            self.state.num_bt_sent@,
            self.might_be_done,
        ) }
    }
}

impl Invariant for CobsEncoder<'_> {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! {
            1 <= self@.3 && self@.3 <= 254
                && self@.2 <= self@.1 && self@.1 <= self@.0 + 1
        }
    }
}

impl<'a> CobsEncoder<'a> {
    /// Create a new streaming Cobs Encoder.
    #[ensures(result@ == (out_buf@.len(), 1, 0, 1, false))]
    #[ensures(inv(result))]
    pub fn new(out_buf: &'a mut [u8]) -> CobsEncoder<'a> {
        CobsEncoder {
            dest: out_buf,
            dest_idx: 1,
            state: EncoderState::default(),
            might_be_done: false,
        }
    }

    /// Push a slice of data to be encoded
    #[trusted]
    #[requires(inv(*self))]
    #[ensures(inv(^self))]
    #[ensures(match result { Ok(()) => true, Err(_) => (^self)@.1 >= (*self)@.1 })]
    pub fn push(&mut self, data: &[u8]) -> Result<(), DestBufTooSmallError> {
        // TODO: could probably check if this would fit without
        // iterating through all data

        // There was the possibility that the encoding process is done, but more data is pushed
        // instead of a `finalize` call, so the destination index needs to be incremented.
        if self.might_be_done {
            self.dest_idx += 1;
            self.might_be_done = false;
        }
        for (slice_idx, val) in data.iter().enumerate() {
            use PushResult::*;
            match self.state.push(*val) {
                AddSingle(y) => {
                    *self
                        .dest
                        .get_mut(self.dest_idx)
                        .ok_or(DestBufTooSmallError)? = y;
                }
                ModifyFromStartAndSkip((idx, mval)) => {
                    *self.dest.get_mut(idx).ok_or(DestBufTooSmallError)? = mval;
                }
                ModifyFromStartAndPushAndSkip((idx, mval, nval1)) => {
                    *self.dest.get_mut(idx).ok_or(DestBufTooSmallError)? = mval;
                    *self
                        .dest
                        .get_mut(self.dest_idx)
                        .ok_or(DestBufTooSmallError)? = nval1;
                    // Do not increase index if these is the possibility that we are finished.
                    if slice_idx == data.len() - 1 {
                        // If push is called again, the index will be incremented. If finalize
                        // is called, there is no need to increment the index.
                        self.might_be_done = true;
                    } else {
                        self.dest_idx += 1;
                    }
                }
            }

            // All branches above require advancing the pointer at least once
            self.dest_idx += 1;
        }

        Ok(())
    }

    /// Complete encoding of the output message. Does NOT terminate the message with the sentinel
    /// value.
    #[trusted]
    #[requires(inv(self))]
    #[ensures(result@ <= self@.0 + 1)]
    pub fn finalize(self) -> usize {
        // Get the last index that needs to be fixed
        let (idx, mval) = if self.dest_idx == 0 {
            (0, 0x01)
        } else {
            self.state.finalize()
        };

        // If the current code index is outside of the destination slice,
        // we do not need to write it out
        if let Some(i) = self.dest.get_mut(idx) {
            *i = mval;
        }

        self.dest_idx
    }
}

/// Encodes the `source` buffer into the `dest` buffer.
///
/// This function assumes the typical sentinel value of 0, but does not terminate the encoded
/// message with the sentinel value. This should be done by the caller to ensure proper framing.
///
/// # Returns
///
/// The number of bytes written to in the `dest` buffer.
///
/// # Panics
///
/// This function will panic if the `dest` buffer is not large enough for the
/// encoded message. You can calculate the size the `dest` buffer needs to be with
/// the [crate::max_encoding_length] function.
#[trusted]
#[requires(dest@.len() >= crate::cobs_encode_model(source@, 0u8).len())]
#[ensures(result@ == crate::cobs_encode_model(source@, 0u8).len())]
#[ensures((^dest)@.subsequence(0, result@) == crate::cobs_encode_model(source@, 0u8))]
#[ensures(forall<i: Int> result@ <= i && i < dest@.len() ==> (^dest)[i] == (*dest)[i])]
pub fn encode(source: &[u8], dest: &mut [u8]) -> usize {
    let mut enc = CobsEncoder::new(dest);
    #[cfg(creusot)]
    if enc.push(source).is_err() {
        panic!("destination buffer too small");
    }
    #[cfg(not(creusot))]
    enc.push(source).unwrap();
    enc.finalize()
}

/// Encodes the `source` buffer into the `dest` buffer, including the default sentinel values 0
/// around the encoded frame.
///
/// # Returns
///
/// The number of bytes written to in the `dest` buffer.
///
/// # Panics
///
/// This function will panic if the `dest` buffer is not large enough for the
/// encoded message. You can calculate the size the `dest` buffer needs to be by adding
/// the [crate::max_encoding_length] function output and 2.
#[trusted]
#[requires(dest@.len() >= crate::cobs_encode_model(source@, 0u8).len() + 2)]
#[ensures(result@ == crate::cobs_encode_model(source@, 0u8).len() + 2)]
#[ensures((^dest)[0] == 0u8 && (^dest)[result@ - 1] == 0u8)]
#[ensures((^dest)@.subsequence(1, result@ - 1) == crate::cobs_encode_model(source@, 0u8))]
#[ensures(forall<i: Int> result@ <= i && i < dest@.len() ==> (^dest)[i] == (*dest)[i])]
pub fn encode_including_sentinels(source: &[u8], dest: &mut [u8]) -> usize {
    if dest.len() < 2 {
        panic!("destination buffer too small");
    }

    dest[0] = 0;
    let mut enc = CobsEncoder::new(&mut dest[1..]);
    #[cfg(creusot)]
    if enc.push(source).is_err() {
        panic!("destination buffer too small");
    }
    #[cfg(not(creusot))]
    enc.push(source).unwrap();
    let encoded_len = enc.finalize();
    dest[encoded_len + 1] = 0;
    encoded_len + 2
}

/// Attempts to encode the `source` buffer into the `dest` buffer.
///
/// This function assumes the typical sentinel value of 0, but does not terminate the encoded
/// message with the sentinel value. This should be done by the caller to ensure proper framing.
///
/// # Returns
///
/// The number of bytes written to in the `dest` buffer.
///
/// If the destination buffer does not have enough room, an error will be returned.
#[trusted]
#[ensures(match result {
    Ok(count) => count@ == crate::cobs_encode_model(source@, 0u8).len()
        && (^dest)@.subsequence(0, count@) == crate::cobs_encode_model(source@, 0u8),
    Err(_) => dest@.len() < crate::cobs_encode_model(source@, 0u8).len(),
})]
pub fn try_encode(source: &[u8], dest: &mut [u8]) -> Result<usize, DestBufTooSmallError> {
    let mut enc = CobsEncoder::new(dest);
    enc.push(source)?;
    Ok(enc.finalize())
}

/// Encodes the `source` buffer into the `dest` buffer, including the default sentinel values 0
/// around the encoded frame.
///
/// # Returns
///
/// The number of bytes written to in the `dest` buffer.
///
/// If the destination buffer does not have enough room, an error will be returned.
#[trusted]
#[ensures(match result {
    Ok(count) => count@ == crate::cobs_encode_model(source@, 0u8).len() + 2
        && (^dest)[0] == 0u8 && (^dest)[count@ - 1] == 0u8
        && (^dest)@.subsequence(1, count@ - 1) == crate::cobs_encode_model(source@, 0u8),
    Err(_) => dest@.len() < crate::cobs_encode_model(source@, 0u8).len() + 2,
})]
pub fn try_encode_including_sentinels(
    source: &[u8],
    dest: &mut [u8],
) -> Result<usize, DestBufTooSmallError> {
    if dest.len() < 2 {
        return Err(DestBufTooSmallError);
    }
    dest[0] = 0;
    let mut enc = CobsEncoder::new(&mut dest[1..]);
    enc.push(source)?;
    let encoded_len = enc.finalize();
    dest[encoded_len + 1] = 0;
    Ok(encoded_len + 2)
}

/// Encodes the `source` buffer into the `dest` buffer using an
/// arbitrary sentinel value.
///
/// This is done by first encoding the message with the typical sentinel value
/// of 0, then XOR-ing each byte of the encoded message with the chosen sentinel
/// value. This will ensure that the sentinel value doesn't show up in the encoded
/// message. See the paper "Consistent Overhead Byte Stuffing" for details.
///
/// This function does not terminate the encoded message with the sentinel value. This should be
/// done by the caller to ensure proper framing.
///
/// # Returns
///
/// The number of bytes written to in the `dest` buffer.
#[trusted]
#[requires(dest@.len() >= crate::cobs_encode_model(source@, sentinel).len())]
#[ensures(result@ == crate::cobs_encode_model(source@, sentinel).len())]
#[ensures((^dest)@.subsequence(0, result@) == crate::cobs_encode_model(source@, sentinel))]
#[ensures(forall<i: Int> result@ <= i && i < dest@.len() ==> (^dest)[i] == (*dest)[i])]
pub fn encode_with_sentinel(source: &[u8], dest: &mut [u8], sentinel: u8) -> usize {
    let encoded_size = encode(source, dest);
    for x in &mut dest[..encoded_size] {
        *x ^= sentinel;
    }
    encoded_size
}

#[cfg(feature = "alloc")]
/// Encodes the `source` buffer into a vector, using the [encode] function.
#[trusted]
#[ensures(result@ == crate::cobs_encode_model(source@, 0u8))]
pub fn encode_vec(source: &[u8]) -> alloc::vec::Vec<u8> {
    let mut encoded = alloc::vec![0; crate::max_encoding_length(source.len())];
    let encoded_len = encode(source, &mut encoded[..]);
    encoded.truncate(encoded_len);
    encoded
}

#[cfg(feature = "alloc")]
/// Encodes the `source` buffer into a vector, using the [encode] function, while also adding
/// the sentinels around the encoded frame.
#[trusted]
#[ensures(result@.len() >= crate::cobs_encode_model(source@, 0u8).len() + 2)]
#[ensures(result@[0] == 0u8)]
#[ensures(result@.subsequence(1, 1 + crate::cobs_encode_model(source@, 0u8).len())
    == crate::cobs_encode_model(source@, 0u8))]
#[ensures(result@[1 + crate::cobs_encode_model(source@, 0u8).len()] == 0u8)]
pub fn encode_vec_including_sentinels(source: &[u8]) -> alloc::vec::Vec<u8> {
    let mut encoded = alloc::vec![0; crate::max_encoding_length(source.len()) + 2];
    let encoded_len = encode_including_sentinels(source, &mut encoded);
    encoded.truncate(encoded_len + 2);
    encoded
}

#[cfg(feature = "alloc")]
/// Encodes the `source` buffer into a vector with an arbitrary sentinel value, using the
/// [encode_with_sentinel] function.
#[trusted]
#[ensures(result@ == crate::cobs_encode_model(source@, sentinel))]
pub fn encode_vec_with_sentinel(source: &[u8], sentinel: u8) -> alloc::vec::Vec<u8> {
    let mut encoded = alloc::vec![0; crate::max_encoding_length(source.len())];
    let encoded_len = encode_with_sentinel(source, &mut encoded[..], sentinel);
    encoded.truncate(encoded_len);
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        decode_vec,
        tests::{test_decode_in_place, test_encode_decode_free_functions},
    };

    #[test]
    fn test_encode_0() {
        // An empty input is encoded as no characters.
        let mut output = [0xFFu8; 16];
        let used = encode(&[], &mut output);
        assert_eq!(used, 1);
        assert_eq!(output[0], 0x01);
    }

    fn test_pair(source: &[u8], encoded: &[u8]) {
        test_encode_decode_free_functions(source, encoded);
        test_decode_in_place(source, encoded);
    }

    #[test]
    fn test_encode_1() {
        test_pair(&[10, 11, 0, 12], &[3, 10, 11, 2, 12])
    }

    #[test]
    fn test_encode_empty() {
        test_pair(&[], &[1])
    }

    #[test]
    fn test_encode_2() {
        test_pair(&[0, 0, 1, 0], &[1, 1, 2, 1, 1])
    }

    #[test]
    fn test_encode_3() {
        test_pair(&[255, 0], &[2, 255, 1])
    }

    #[test]
    fn test_encode_4() {
        test_pair(&[1], &[2, 1])
    }

    #[test]
    fn encode_target_buf_too_small() {
        let source = &[10, 11, 0, 12];
        let expected = &[3, 10, 11, 2, 12];
        for len in 0..expected.len() {
            let mut dest = alloc::vec![0; len];
            matches!(
                try_encode(source, &mut dest).unwrap_err(),
                DestBufTooSmallError
            );
        }
    }

    #[test]
    fn try_encode_with_sentinels() {
        let source = &[10, 11, 0, 12];
        let expected = &[0, 3, 10, 11, 2, 12, 0];
        let mut dest = alloc::vec![0; expected.len()];
        let encoded_len = try_encode_including_sentinels(source, &mut dest).unwrap();
        assert_eq!(encoded_len, expected.len());
        assert_eq!(dest[0], 0);
        assert_eq!(dest[expected.len() - 1], 0);
        assert_eq!(decode_vec(&dest).unwrap(), source);
    }

    #[test]
    fn test_encoding_including_sentinels() {
        let data = [1, 2, 3];
        let encoded = encode_vec_including_sentinels(&data);
        assert_eq!(*encoded.first().unwrap(), 0);
        assert_eq!(*encoded.last().unwrap(), 0);
        let data_decoded = decode_vec(&encoded).unwrap();
        assert_eq!(data_decoded, data);
        let data_decoded = decode_vec(&encoded[1..]).unwrap();
        assert_eq!(data_decoded, data);
        let data_decoded = decode_vec(&encoded[1..encoded.len() - 1]).unwrap();
        assert_eq!(data_decoded, data);
    }

    #[test]
    #[should_panic]
    fn encode_target_buf_too_small_panicking() {
        let source = &[10, 11, 0, 12];
        let expected = &[3, 10, 11, 2, 12];
        encode(source, &mut alloc::vec![0; expected.len() - 1]);
    }
}
