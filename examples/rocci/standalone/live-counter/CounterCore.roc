import Origin

CounterCore := [].{
    Click : { seq : I64, ts : I64, tz : Str, chosen : Bool }
    State : { count : I64, revision : I64, clicks : List(Click) }
    Command : [Increment({ origin : Str, tz : Str }), Reset]

    # The private JS host uses exact integers within this common range.
    max_integer : I64
    max_integer = 9007199254740991

    initial : State
    initial = { count: 0, revision: 0, clicks: [] }

    view = |state, now| {
        count: state.count,
        clicks: List.map(state.clicks, |click| {
            place = Origin.lookup(click.tz)
            { flag: place.flag, place: Origin.place_label(place), ago: Origin.relative_ago(now, click.ts), ts: click.ts.to_str(), chosen: click.chosen }
        }),
    }

    decide : State, Command, I64 -> Try(State, [Overflow])
    decide = |state, command, now| {
        if state.revision >= max_integer {
            Err(Overflow)
        } else {
            revision = state.revision + 1
            match command {
                Reset => Ok({ ..state, count: 0, revision })
                Increment({ origin, tz: browser_tz }) => {
                    if state.count >= max_integer {
                        Err(Overflow)
                    } else {
                        chosen = origin != ""
                        tz = match (chosen, browser_tz) {
                            (True, _) => origin
                            (False, "") => "UTC"
                            (False, value) => value
                        }
                        click = { seq: revision, ts: now, tz, chosen }
                        clicks = List.concat([click], state.clicks)
                        retained = if clicks.len() > 50 { List.drop_last(clicks, clicks.len() - 50) } else { clicks }
                        Ok({ count: state.count + 1, revision, clicks: retained })
                    }
                }
            }
        }
    }
}
