use rust_icu_sys as sys;
use rust_icu_ubrk as ubrk;

pub struct WordBreaker {
    break_iterator: ubrk::UBreakIterator,
    /*
    int32_t iteratorNext();
    void detectEmailOrUrl();
    ssize_t findNextBreakInEmailOrUrl();

    std::unique_ptr<icu::BreakIterator> mBreakIterator;
    UText m_utext = UTEXT_INITIALIZER;
    const uint16_t* mText = nullptr;
    size_t mTextSize;
    ssize_t mLast;
    ssize_t mCurrent;
    bool mIteratorWasReset;

    // state for the email address / url detector
    ssize_t mScanOffset;
    bool mInEmailOrUrl;
    */
}

impl WordBreaker {
    pub fn new(text: &str) -> Self {}
    /*
        //~WordBreaker() { finish(); }

        // libtxt extension: always use the default locale so that a cached instance
        // of the ICU break iterator can be reused.
        fn set_locale();

        fn set_text(data: &str);

        // Advance iterator to next word break. Return offset, or -1 if EOT
        fn next() -> Option<usize>;

        // Current offset of iterator, equal to 0 at BOT or last return from next()
        const fn current() -> isize;

        /// After calling next(), word_start() and word_end() are offsets defining the previous word.
        /// If wordEnd <= wordStart, it's not a word for the purpose of hyphenation.
        const fn word_start() -> isize;

        const fn word_end() -> isize;

        const fn break_badness() -> i32;

        const fn finish();
    */
}
