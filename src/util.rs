/// Removes special characters from a song name so that it can be used in a filename
///
/// # Examples
///
/// ```
/// let song_name = "(Dark/\\ //Star-/-->";
///
/// assert_eq!("Dark Star", sanitize_song_name(song_name));
/// ```
///
/// # Args
///
/// 1. `s` - string slice to sanitize
///
pub fn sanitize_song_name(s: &str) -> String {
  String::from(
    s.replace(
      &[
        '(', ')', ',', '\"', '.', ';', ':', '\'', '\\', '/', '-', '>', '<',
      ][..],
      "",
    )
    .trim(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check_dark_star() {
    let song_name = "(Dark/\\ //Star-/-->";

    assert_eq!("Dark Star", sanitize_song_name(song_name));
  }

  #[test]
  fn check_trim() {
    let song_name = "  The .,:;Music ->Never <-Stopped-->  ";
    assert_eq!("The Music Never Stopped", sanitize_song_name(song_name));
  }
}
