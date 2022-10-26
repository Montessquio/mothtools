use nom::IResult;
use nom::complete::tag;

enum KeywordToken {
    Aspect,
    Card,
    Hidden,
    Unique,
    Mansus,
    Deck,
    Craft,
    Hint,
    Verb,
    Temp,
    Slot,
    Consume,
    Greedy,
    Decay,
    Spawn,
    XTrigger, // Unsure Spec
    Aspects,
    Warmup,
    Apply,
    Draw,
    Signal,
    Purge,
    Burn,
    Portal,
    Ending,
    Halt,
    Delete,
    Goto,
    If,
}

enum TransitionTokens {
    /// {
    OpenBrace,
    /// }
    CloseBrace,
    /// [
    OpenBracket,
    /// ]
    CloseBracket,
    /// (
    OpenParen,
    /// )
    CloseParen,
    /// :
    Colon,
    /// ,
    Comma,
    /// ->
    Arrow,
    /// \n
    Newline,
    /// =
    Equals,
    /// -=
    MinusEquals,
    /// +=
    PlusEquals,
    /// ?
    Question,
    /// !
    Bang,
}

enum ValueToken {
    Comment(String),
    String(String),
    Number(i32),
}

/* Tokens */
fn aspect(i: &str) -> IResult<&str, KeywordToken>{
    tag("HTTP/")(i)
}
