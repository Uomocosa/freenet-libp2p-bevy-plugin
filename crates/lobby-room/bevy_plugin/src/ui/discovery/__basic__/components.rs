use crate::discovery as core_discovery;
use bevy::prelude::Component;
use derive_more::{Deref, DerefMut};

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Root;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomNameInput;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct CreateRoomButton;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomList;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomNameClip;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomNameScroller;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomBubble;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomFriendSegment;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomBubbleTooltip;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
pub struct RoomButton(pub core_discovery::RoomName);

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
pub struct EnterRoomButton(pub core_discovery::RoomName);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomFullBadge;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomPreviewPanel;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomPreviewViewport;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomPreviewPlaceholder;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomPreviewImage;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomStreamPanel;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomStreamHeader;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomStreamViewport;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomStreamImage;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomStreamPlaceholder;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomStreamName;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomStreamOccupancyBubble;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomStreamOccupancyText;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomStreamActionSlot;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomStreamFullscreenButton;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomStreamCloseButton;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LeaveDoorRoot;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LeaveDoor;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct DoorOpening(pub f32);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DoorPanel;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DoorLabel;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LeaveYesButton;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LeaveNoButton;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct HoverGlow(pub f32);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct HoverOutlineOnly;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct CreateShake(pub f32);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ConnectingRoot;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SpinnerRing;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomsHeader;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomSearchIndicator;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomSearchTooltip;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct JoiningRoomRoot;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct JoinErrorRoot;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct JoinErrorDismissButton;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct EscMenu;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct EscMenuBackdrop;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct EscMenuFriendsButton;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct EscMenuLeaveButton;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct EscMenuResumeButton;
