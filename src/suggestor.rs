#[derive(Debug, Clone, Copy)]
pub enum SuggestionKind {
    Command,
    SubCommand,
    Keyword,
    Column,
    Table,
}

#[derive(Debug)]
pub struct Suggestion {
    pub display: String,
    pub kind: SuggestionKind,
}

pub struct CompletionContext<'a> {
    pub tables: &'a [&'a str],
    pub columns: &'a [&'a str],
}

pub fn suggest(ctx: &CompletionContext, line: &str) -> Vec<Suggestion> {
    let ends_in_space = line.chars().last().is_some_and(|c| c.is_whitespace());
    let tokens: Vec<&str> = line.split_whitespace().collect();

    // Decide which slot we're completing and the partial token typed so far
    let (source, partial): (Source, &str) = if tokens.is_empty() || (tokens.len() == 1 && !ends_in_space) {
        // Nothing yet, or still typing the first command
        (Source::Commands, tokens.first().copied().unwrap_or(""))
    } else {
        // `partial` is the token under the cursor and `arg_index` counts the
        // argument tokens committed before it. This positional index is what lets
        // context advance as more tokens are typed, rather than keying off the command word alone.

        let partial = if ends_in_space { "" } else { *tokens.last().unwrap() };

        let arg_index = if ends_in_space {
            tokens.len() - 1
        } else {
            tokens.len() - 2
        };

        match resolve_source(tokens[0], arg_index, &tokens) {
            Some(source) => (source, partial),
            None => return Vec::new(),
        }
    };

    return expand(source, ctx)
        .into_iter()
        .filter(|suggestion| fuzzy_matches(partial, &suggestion.display))
        .collect();
}

/// Resolves the `Source` feeding suggestions for the argument at `arg_index` of
/// `command`. Returns `None` when the command is unknown or the slot expects a
/// free value we can't suggest (e.g. a number or a `where` operand).
fn resolve_source(command: &str, arg_index: usize, tokens: &[&str]) -> Option<Source> {
    let spec = COMMANDS.iter().find(|spec| spec.names.contains(&command))?;

    // `goto page <n>` branches off the table list into a bare page number.
    if spec.names[0] == "goto" && tokens.get(1) == Some(&"page") && arg_index >= 1 {
        return None;
    }

    return match spec.slots.get(arg_index)? {
        Source::Commands => Some(Source::Commands),
        Source::Keywords(keywords) => Some(Source::Keywords(keywords)),
        Source::Tables => Some(Source::Tables),
        Source::Columns => Some(Source::Columns),
        Source::TableOrKeyword(keywords) => Some(Source::TableOrKeyword(keywords)),
        Source::SettingKey(keys) => Some(Source::SettingKey(keys)),
    };
}

/// Expands a `Source` into its full candidate list, before fuzzy filtering.
fn expand(source: Source, ctx: &CompletionContext) -> Vec<Suggestion> {
    let make = |display: &str, kind| Suggestion { display: display.to_string(), kind };

    return match source {
        Source::Commands => COMMANDS
            .iter()
            .map(|spec| make(spec.names[0], SuggestionKind::Command))
            .collect(),
        Source::Keywords(kws) => kws.iter().map(|k| make(k, SuggestionKind::Keyword)).collect(),
        Source::Tables => ctx.tables.iter().map(|t| make(t, SuggestionKind::Table)).collect(),
        Source::Columns => ctx.columns.iter().map(|c| make(c, SuggestionKind::Column)).collect(),
        Source::TableOrKeyword(kws) => kws
            .iter()
            .map(|k| make(k, SuggestionKind::SubCommand))
            .chain(ctx.tables.iter().map(|t| make(t, SuggestionKind::Table)))
            .collect(),
        Source::SettingKey(keys) => keys.iter().map(|k| make(k, SuggestionKind::Keyword)).collect(),
    };
}

/// Case-insensitive subsequence match: every char of `input` appears in
/// `candidate` in order. Empty input matches everything.
fn fuzzy_matches(input: &str, candidate: &str) -> bool {
    let mut input = input.chars().map(|c| c.to_ascii_lowercase());
    let mut current = input.next();

    for ch in candidate.chars().map(|c| c.to_ascii_lowercase()) {
        match current {
            None => return true,
            Some(expected) if ch == expected => current = input.next(),
            _ => {}
        }
    }

    return current.is_none();
}

enum Source {
    Commands,
    Keywords(&'static [&'static str]),
    Tables,
    Columns,
    TableOrKeyword(&'static [&'static str]),
    SettingKey(&'static [&'static str]),
}

struct Spec {
    names: &'static [&'static str],
    // What we should suggest. Empty for no suggestions like a number.
    slots: &'static [Source],
}

// @Commands
/// names.0 is canonical (what is display); rest = aliases (so `w`/`ob`/`g` resolve)
const COMMANDS: &[Spec] = &[
    Spec { names: &["quit", "q"], slots: &[] },
    Spec { names: &["count", "c"], slots: &[] },
    Spec { names: &["total-count", "tc"], slots: &[] },
    Spec {
        names: &["goto", "g"],
        slots: &[Source::TableOrKeyword(&["page"])],
    },
    Spec { names: &["order-by", "ob"], slots: &[Source::Columns] },
    Spec { names: &["where", "w"], slots: &[Source::Columns] },
    Spec { names: &["limit", "l"], slots: &[] }, // number, no candidates
    Spec { names: &["refresh"], slots: &[] },
    Spec { names: &["set"], slots: &[Source::SettingKey(SETTINGS)] },
    Spec { names: &["help"], slots: &[] },
];

// @Settings
const SETTINGS: &[&'static str] = &["transparent_background", "default_limit", "default_sort"];
