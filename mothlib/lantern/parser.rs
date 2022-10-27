

fn peol_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E>
{
  value(
    (), // Output is thrown away.
    pair(char('%'), is_not("\n\r"))
  )(i)
}
