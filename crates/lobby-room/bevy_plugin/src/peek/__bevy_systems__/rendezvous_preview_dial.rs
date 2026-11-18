use crate::discovery;
use crate::identity;

#[must_use]
pub(crate) fn rendezvous_preview_dial<'a>(
    me: Option<&identity::PeerId>,
    members: impl IntoIterator<Item = &'a discovery::RoomMember>,
) -> Option<&'a discovery::RoomMember> {
    let mut dialable: Vec<&'a discovery::RoomMember> = Vec::new();
    let mut occupants: Vec<&'a discovery::RoomMember> = Vec::new();
    for member in members {
        if member.peer.addrs.is_empty() {
            continue;
        }
        if member.membership == discovery::RoomMembership::Occupant {
            occupants.push(member);
        } else {
            dialable.push(member);
        }
    }
    if dialable.is_empty() {
        dialable = occupants;
    }
    let Some(me) = me else {
        return dialable.into_iter().next();
    };
    dialable
        .into_iter()
        .filter(|member| member.peer.id != *me)
        .map(|member| {
            let local = me.as_str();
            let remote = member.peer.id.as_str();
            let combined = identity::PeerId::from(format!("{local}:{remote}"));
            (combined.peer_author(), member)
        })
        .max_by(|(one_score, one), (other_score, other)| {
            one_score
                .cmp(other_score)
                .then_with(|| other.peer.id.cmp(&one.peer.id))
        })
        .map(|(_, pick)| pick)
}
