use source::Span;
use fixity::{Prec, HasPrec, NO_PREC};
use item::GroupItem;
use item::LaxItem;
use item::LaxItem::*;


pub fn lax<'s, 'g>(item: GroupItem<'s, 'g>) -> LaxItem<'s, 'g> {
    match item {
        GroupItem::Lexeme(span, token) =>
            Lexeme(span, token),
        GroupItem::Multeme(span, op, groups) => {
            Multeme(span, op, lax_groups(groups))
        }
    }
}

fn lax_groups<'s, 'g>(groups: Vec<(Span<'s>, Vec<GroupItem<'s, 'g>>)>)
                      -> Vec<(Span<'s>, Vec<LaxItem<'s, 'g>>)> {
    groups.into_iter().map(|(span, items)| {
        (span, lax_group(span, items, true))
    }).collect()
}

fn lax_group<'s, 'g>(span: Span<'s>,
                     mut items: Vec<GroupItem<'s, 'g>>,
                     mut needy: bool)
                     -> Vec<LaxItem<'s, 'g>> {
    let mut result: Vec<LaxItem<'s, 'g>> = vec!();

                if !needy {
                    result.push(Juxt(span.start()));
                }
                result.push(Lexeme(span, token));
                needy = false;

    while let Some(item) = items.pop() {
        let item = match item {
            GroupItem::Lexeme(span, token) =>
                Lexeme(span, token),
            GroupItem::Multeme(span, op, groups) =>
                Multeme(span, op, lax_groups(groups));
        };
        
    }
    if !needy {
        result.push(Juxt(span.end()));
    }
    result
}
