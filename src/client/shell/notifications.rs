#[cfg(test)]
use super::notification_policy::COMPLETION_RECHECK_INTERVAL;
use super::*;
use ratatui::{
    style::Color,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

pub(super) fn render_mobile_notice_banner(
    buffer: &mut Buffer,
    area: Rect,
    title: &str,
    body: Option<&str>,
    dot_color: Color,
    offset_for_warning: bool,
    palette: &Palette,
) -> Rect {
    if area.is_empty() {
        return Rect::default();
    }
    let warning_offset = u16::from(offset_for_warning);
    let y = area.y
        + area
            .height
            .saturating_sub(1u16.saturating_add(warning_offset));
    let rect = Rect::new(area.x, y, area.width, 1);
    let background = palette.surface0;
    Clear.render(rect, buffer);
    buffer.set_style(rect, Style::default().bg(background));
    let mut x = rect.x;
    for (text, style) in [
        (" ", Style::default().bg(background)),
        ("●", Style::default().fg(dot_color).bg(background)),
        (" ", Style::default().bg(background)),
        (
            title,
            Style::default()
                .fg(palette.text)
                .bg(background)
                .add_modifier(Modifier::BOLD),
        ),
    ] {
        x = super::render::put_segment(buffer, x, rect.y, rect.right(), text, style);
    }
    if let Some(body) = body.filter(|body| !body.is_empty()) {
        x = super::render::put_segment(
            buffer,
            x,
            rect.y,
            rect.right(),
            " · ",
            Style::default().fg(palette.overlay0).bg(background),
        );
        super::render::put_text(
            buffer,
            x,
            rect.y,
            rect.right().saturating_sub(x),
            body,
            Style::default().fg(palette.overlay0).bg(background),
        );
    }
    rect
}

pub(super) fn render_mobile_notification_banner(
    buffer: &mut Buffer,
    area: Rect,
    notification: &ClientVisibleNotification,
    offset_for_warning: bool,
    palette: &Palette,
) -> Rect {
    let event = &notification.event;
    let title = match event.kind {
        SemanticNotificationKind::NeedsAttention => event
            .title
            .strip_suffix(" needs attention")
            .map(|agent| format!("{agent} waiting"))
            .unwrap_or_else(|| event.title.clone()),
        SemanticNotificationKind::Finished => event
            .title
            .strip_suffix(" finished")
            .map(|agent| format!("{agent} done"))
            .unwrap_or_else(|| event.title.clone()),
        SemanticNotificationKind::UpdateInstalled => "update ready".to_owned(),
        SemanticNotificationKind::Custom => event.title.clone(),
    };
    let dot_color = match event.kind {
        SemanticNotificationKind::NeedsAttention => palette.red,
        SemanticNotificationKind::Finished => palette.blue,
        SemanticNotificationKind::UpdateInstalled | SemanticNotificationKind::Custom => {
            palette.accent
        }
    };
    render_mobile_notice_banner(
        buffer,
        area,
        &title,
        event.body.as_deref(),
        dot_color,
        offset_for_warning,
        palette,
    )
}

pub(super) fn render_notification_card(
    buffer: &mut Buffer,
    area: Rect,
    title: &str,
    body: &str,
    position: crate::config::ToastHerdrPosition,
    top_offset: u16,
    dot_color: Color,
    palette: &Palette,
) -> Rect {
    if area.is_empty() {
        return Rect::default();
    }
    let content_width = unicode_width::UnicodeWidthStr::width(title)
        .max(unicode_width::UnicodeWidthStr::width(body))
        .saturating_add(6);
    let width = u16::try_from(content_width)
        .unwrap_or(u16::MAX)
        .min(area.width);
    let height: u16 = if body.is_empty() { 3 } else { 4 }.min(area.height);
    let x = match position {
        crate::config::ToastHerdrPosition::TopLeft
        | crate::config::ToastHerdrPosition::BottomLeft => area.x,
        crate::config::ToastHerdrPosition::TopRight
        | crate::config::ToastHerdrPosition::BottomRight => area.right().saturating_sub(width),
    };
    let max_y = area.bottom().saturating_sub(height).max(area.y);
    let y = match position {
        crate::config::ToastHerdrPosition::TopLeft
        | crate::config::ToastHerdrPosition::TopRight => area.y.saturating_add(top_offset),
        crate::config::ToastHerdrPosition::BottomLeft
        | crate::config::ToastHerdrPosition::BottomRight => area
            .bottom()
            .saturating_sub(height.saturating_add(top_offset)),
    }
    .clamp(area.y, max_y);
    let rect = Rect::new(x, y, width, height);
    Clear.render(rect, buffer);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.overlay0))
        .style(Style::default().bg(palette.panel_bg));
    let inner = block.inner(rect);
    block.render(rect, buffer);
    Paragraph::new(Line::from(vec![
        Span::styled("●", Style::default().fg(dot_color)),
        Span::raw(" "),
        Span::styled(
            title,
            Style::default()
                .fg(palette.text)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .render(Rect::new(inner.x, inner.y, inner.width, 1), buffer);
    if !body.is_empty() && inner.height > 1 {
        Paragraph::new(Line::from(Span::styled(
            body,
            Style::default().fg(palette.overlay0),
        )))
        .render(
            Rect::new(
                inner.x.saturating_add(2),
                inner.y + 1,
                inner.width.saturating_sub(2),
                1,
            ),
            buffer,
        );
    }
    rect
}

pub(super) fn render_visible_notification(
    buffer: &mut Buffer,
    area: Rect,
    notification: &ClientVisibleNotification,
    default_position: crate::config::ToastHerdrPosition,
    top_offset: u16,
    palette: &Palette,
) -> Rect {
    let event = &notification.event;
    let dot_color = match event.kind {
        SemanticNotificationKind::NeedsAttention => palette.red,
        SemanticNotificationKind::Finished => palette.blue,
        SemanticNotificationKind::UpdateInstalled | SemanticNotificationKind::Custom => {
            palette.accent
        }
    };
    render_notification_card(
        buffer,
        area,
        &event.title,
        event.body.as_deref().unwrap_or_default(),
        event.position.unwrap_or(default_position),
        top_offset,
        dot_color,
        palette,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notification() -> ClientVisibleNotification {
        ClientVisibleNotification {
            endpoint_id: ClientEndpointId::Local,
            event: SemanticNotification {
                kind: SemanticNotificationKind::Custom,
                title: "notice".into(),
                body: None,
                sound: None,
                agent: None,
                workspace_id: None,
                tab_id: None,
                pane_id: None,
                position: None,
            },
            deadline: std::time::Instant::now(),
        }
    }

    #[test]
    fn mobile_notification_is_a_bottom_banner_with_released_title() {
        let palette = crate::app::client_palette_from_config(&Config::default());
        let mut notification = notification();
        notification.event.kind = SemanticNotificationKind::NeedsAttention;
        notification.event.title = "pi needs attention".into();
        notification.event.body = Some("workspace · tab 1".into());
        let area = Rect::new(0, 0, 44, 20);
        let mut buffer = Buffer::empty(area);
        for cell in &mut buffer.content {
            cell.set_symbol("X");
        }
        let rect =
            render_mobile_notification_banner(&mut buffer, area, &notification, true, &palette);
        assert_eq!(rect, Rect::new(0, 18, 44, 1));
        let text = buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(text.contains("pi waiting"));
        assert!(text.contains("workspace · tab 1"));
        assert!(buffer.content[18 * 44..19 * 44]
            .iter()
            .all(|cell| cell.symbol() != "X"));
    }

    #[test]
    fn unavailable_notification_target_stays_visible_and_reports_the_machine() {
        let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
        state.visible_notification = Some(ClientVisibleNotification {
            endpoint_id: ClientEndpointId::Local,
            event: SemanticNotification {
                kind: SemanticNotificationKind::NeedsAttention,
                title: "agent needs attention".into(),
                body: None,
                sound: None,
                agent: Some("agent".into()),
                workspace_id: Some("workspace".into()),
                tab_id: Some("tab".into()),
                pane_id: Some("pane".into()),
                position: None,
            },
            deadline: std::time::Instant::now() + std::time::Duration::from_secs(5),
        });
        let mut outcome = ClientShellInput::default();

        state.focus_visible_notification(&mut outcome);

        assert!(state.visible_notification.is_some());
        assert!(state
            .visible_endpoint_notice
            .as_ref()
            .is_some_and(|notice| notice.body.contains("Local is unavailable")));
        assert!(outcome.repaint);
    }

    #[test]
    fn immediate_finished_evidence_for_acknowledged_idle_never_toasts_or_sounds() {
        let mut config = ClientShellConfig::from_config(&Config::default());
        config.toast_delivery = crate::config::ToastDelivery::Herdr;
        config.toast_delay_seconds = 0;
        let mut state = ClientShellState::new(config);
        let mut snapshot = super::super::tests::snapshot();
        snapshot.agents.push(crate::protocol::ClientShellAgent {
            pane_id: "pane_1".into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            name: Some("agent".into()),
            display_agent: None,
            agent: Some("agent".into()),
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            agent_status: crate::api::schema::AgentStatus::Idle,
            state_change_seq: 2,
            state_labels: Vec::new(),
            tokens: Vec::new(),
            focused: true,
        });
        state.set_snapshot(Box::new(snapshot));

        let (effects, repaint) = state.receive_notification(
            &ClientEndpointId::Local,
            SemanticNotification {
                kind: SemanticNotificationKind::Finished,
                title: "agent finished".into(),
                body: None,
                sound: Some(SemanticNotificationSound::Done),
                agent: Some("agent".into()),
                workspace_id: Some("ws_1".into()),
                tab_id: Some("tab_1".into()),
                pane_id: Some("pane_1".into()),
                position: None,
            },
            std::time::Instant::now(),
        );

        assert!(effects.is_empty());
        assert!(!repaint);
        assert!(state.visible_notification.is_none());
    }

    #[test]
    fn finished_hint_waits_for_the_projected_completion_snapshot() {
        let mut config = ClientShellConfig::from_config(&Config::default());
        config.toast_delivery = crate::config::ToastDelivery::Terminal;
        config.toast_delay_seconds = 0;
        let mut state = ClientShellState::new(config);
        let mut snapshot = super::super::tests::snapshot();
        snapshot.agents.push(crate::protocol::ClientShellAgent {
            pane_id: "pane_1".into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            name: Some("agent".into()),
            display_agent: None,
            agent: Some("agent".into()),
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            agent_status: crate::api::schema::AgentStatus::Working,
            state_change_seq: 1,
            state_labels: Vec::new(),
            tokens: Vec::new(),
            focused: false,
        });
        state.set_snapshot(Box::new(snapshot.clone()));
        let now = std::time::Instant::now();
        let event = SemanticNotification {
            kind: SemanticNotificationKind::Finished,
            title: "agent finished".into(),
            body: None,
            sound: None,
            agent: Some("agent".into()),
            workspace_id: Some("ws_1".into()),
            tab_id: Some("background-tab".into()),
            pane_id: Some("pane_1".into()),
            position: None,
        };

        let (effects, _) = state.receive_notification(&ClientEndpointId::Local, event, now);
        assert!(effects.is_empty());
        assert_eq!(state.pending_notifications.len(), 1);

        snapshot.revision = snapshot.revision.saturating_add(1);
        snapshot.agents[0].agent_status = crate::api::schema::AgentStatus::Idle;
        snapshot.agents[0].state_change_seq = 2;
        state.set_snapshot(Box::new(snapshot));
        let (effects, _) = state.tick_notifications(now + COMPLETION_RECHECK_INTERVAL);
        assert!(matches!(
            effects.as_slice(),
            [ClientShellNotificationEffect::Terminal { title, .. }] if title == "agent finished"
        ));
        assert!(state.pending_notifications.is_empty());
    }

    #[test]
    fn retiring_one_endpoint_preserves_other_endpoint_notifications() {
        let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
        let mut local = notification();
        local.event.title = "local".into();
        let remote_id = ClientEndpointId::Ssh(
            crate::client::endpoint::ProfileId::parse("0123456789abcdef0123456789abcdef").unwrap(),
        );
        let mut remote = notification();
        remote.endpoint_id = remote_id.clone();
        remote.event.title = "remote".into();
        state.visible_notification = Some(local);
        state.queued_notifications.push_back(remote);

        state.retire_endpoint_notifications(&ClientEndpointId::Local);

        assert_eq!(
            state
                .visible_notification
                .as_ref()
                .map(|notification| (&notification.endpoint_id, notification.event.title.as_str())),
            Some((&remote_id, "remote"))
        );
    }

    #[test]
    fn concurrent_endpoint_notifications_are_queued_in_arrival_order() {
        let mut config = Config::default();
        config.ui.toast.delivery = crate::config::ToastDelivery::Herdr;
        let mut state = ClientShellState::new(ClientShellConfig::from_config(&config));
        let now = std::time::Instant::now();
        for title in ["first", "second"] {
            state.receive_notification(
                &ClientEndpointId::Local,
                SemanticNotification {
                    kind: SemanticNotificationKind::Custom,
                    title: title.into(),
                    body: None,
                    sound: None,
                    agent: None,
                    workspace_id: None,
                    tab_id: None,
                    pane_id: Some(title.into()),
                    position: None,
                },
                now,
            );
        }

        assert_eq!(
            state
                .visible_notification
                .as_ref()
                .map(|notification| notification.event.title.as_str()),
            Some("first")
        );
        assert_eq!(state.queued_notifications.len(), 1);

        state.tick_notifications(now + std::time::Duration::from_secs(5));

        assert_eq!(
            state
                .visible_notification
                .as_ref()
                .map(|notification| notification.event.title.as_str()),
            Some("second")
        );
        assert!(state.queued_notifications.is_empty());
    }

    #[test]
    fn notification_rect_stays_inside_short_nonzero_area() {
        let palette = crate::app::client_palette_from_config(&Config::default());
        for height in [1, 2] {
            let area = Rect::new(3, 4, 8, height);
            for position in [
                crate::config::ToastHerdrPosition::TopRight,
                crate::config::ToastHerdrPosition::BottomRight,
            ] {
                let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 10));
                let rect = render_visible_notification(
                    &mut buffer,
                    area,
                    &notification(),
                    position,
                    1,
                    &palette,
                );
                assert!(rect.y >= area.y);
                assert!(rect.bottom() <= area.bottom());
            }
        }
    }

    fn blocked_agent(
        pane_id: &str,
        workspace_id: &str,
        tab_id: &str,
    ) -> crate::protocol::ClientShellAgent {
        crate::protocol::ClientShellAgent {
            pane_id: pane_id.into(),
            workspace_id: workspace_id.into(),
            tab_id: tab_id.into(),
            name: None,
            display_agent: Some("codex".into()),
            agent: Some("codex".into()),
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            agent_status: crate::api::schema::AgentStatus::Blocked,
            state_change_seq: 1,
            state_labels: Vec::new(),
            tokens: Vec::new(),
            focused: false,
        }
    }

    fn attention_snapshot() -> crate::protocol::ClientShellSnapshot {
        let mut snapshot = super::super::tests::snapshot();
        let extra_panes = [("pane_2", "ws_2", "tab_2"), ("pane_3", "ws_3", "tab_3")];
        for (pane_id, workspace_id, tab_id) in extra_panes {
            let mut pane = snapshot.panes[0].clone();
            pane.pane_id = pane_id.into();
            pane.workspace_id = workspace_id.into();
            pane.tab_id = tab_id.into();
            pane.focused = false;
            snapshot.panes.push(pane);

            let mut tab = snapshot.tabs[0].clone();
            tab.tab_id = tab_id.into();
            tab.workspace_id = workspace_id.into();
            tab.focused = false;
            snapshot.tabs.push(tab);

            let mut workspace = snapshot.workspaces[0].clone();
            workspace.workspace_id = workspace_id.into();
            workspace.active_tab_id = tab_id.into();
            workspace.focused = false;
            workspace.number = snapshot.workspaces.len() + 1;
            snapshot.workspaces.push(workspace);
        }
        snapshot.agents = extra_panes
            .iter()
            .map(|(pane_id, workspace_id, tab_id)| blocked_agent(pane_id, workspace_id, tab_id))
            .collect();
        snapshot
    }

    fn ping(pane_id: &str, workspace_id: &str, tab_id: &str) -> SemanticNotification {
        ping_kind(
            SemanticNotificationKind::NeedsAttention,
            pane_id,
            workspace_id,
            tab_id,
        )
    }

    fn ping_finished(pane_id: &str, workspace_id: &str, tab_id: &str) -> SemanticNotification {
        ping_kind(
            SemanticNotificationKind::Finished,
            pane_id,
            workspace_id,
            tab_id,
        )
    }

    fn ping_kind(
        kind: SemanticNotificationKind,
        pane_id: &str,
        workspace_id: &str,
        tab_id: &str,
    ) -> SemanticNotification {
        SemanticNotification {
            kind,
            title: format!("{pane_id} needs attention"),
            body: None,
            sound: None,
            agent: Some("codex".into()),
            workspace_id: Some(workspace_id.into()),
            tab_id: Some(tab_id.into()),
            pane_id: Some(pane_id.into()),
            position: None,
        }
    }

    fn push_attention_pane(
        snapshot: &mut crate::protocol::ClientShellSnapshot,
        pane_id: &str,
        workspace_id: &str,
        tab_id: &str,
    ) {
        let mut pane = snapshot.panes[0].clone();
        pane.pane_id = pane_id.into();
        pane.workspace_id = workspace_id.into();
        pane.tab_id = tab_id.into();
        pane.focused = false;
        snapshot.panes.push(pane);

        let mut tab = snapshot.tabs[0].clone();
        tab.tab_id = tab_id.into();
        tab.workspace_id = workspace_id.into();
        tab.focused = false;
        snapshot.tabs.push(tab);

        let mut workspace = snapshot.workspaces[0].clone();
        workspace.workspace_id = workspace_id.into();
        workspace.active_tab_id = tab_id.into();
        workspace.focused = false;
        workspace.number = snapshot.workspaces.len() + 1;
        snapshot.workspaces.push(workspace);

        snapshot
            .agents
            .push(blocked_agent(pane_id, workspace_id, tab_id));
    }

    fn focused_pane_id(outcome: &ClientShellInput) -> Option<&str> {
        outcome.actions.iter().find_map(|action| match action {
            ClientShellAction::Endpoint { request, .. } => match &request.method {
                crate::api::schema::Method::PaneFocus(target) => Some(target.pane_id.as_str()),
                _ => None,
            },
            _ => None,
        })
    }

    fn attention_state() -> ClientShellState {
        let mut config = ClientShellConfig::from_config(&Config::default());
        config.toast_delivery = crate::config::ToastDelivery::Herdr;
        config.toast_delay_seconds = 0;
        let mut state = ClientShellState::new(config);
        state.set_snapshot(Box::new(attention_snapshot()));
        state
    }

    fn notify_agent(state: &mut ClientShellState, pane_id: &str, workspace_id: &str, tab_id: &str) {
        let now = std::time::Instant::now();
        state.receive_notification(
            &ClientEndpointId::Local,
            ping(pane_id, workspace_id, tab_id),
            now,
        );
    }

    fn queued_pane_ids(state: &ClientShellState) -> Vec<&str> {
        state
            .attention_targets
            .iter()
            .map(|target| target.pane_id.as_str())
            .collect()
    }

    fn record_next_attention(state: &mut ClientShellState) -> ClientShellInput {
        let mut outcome = ClientShellInput::default();
        state.record_binding(
            crate::input::KeybindMatch::Action(crate::input::KeybindAction::NextAttentionTarget),
            &mut outcome,
        );
        outcome
    }

    #[test]
    fn default_prefix_tab_resolves_to_next_attention_target() {
        let config = ClientShellConfig::from_config(&Config::default());
        let tab = crate::input::TerminalKey::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        );
        assert!(matches!(
            crate::input::resolve_prefix_binding(&config.keybinds.keybinds, &tab),
            Some(crate::input::KeybindMatch::Action(
                crate::input::KeybindAction::NextAttentionTarget
            ))
        ));
    }

    #[test]
    fn default_prefix_o_resolves_to_open_notification_target() {
        let config = ClientShellConfig::from_config(&Config::default());
        let o = crate::input::TerminalKey::new(
            crossterm::event::KeyCode::Char('o'),
            crossterm::event::KeyModifiers::empty(),
        );
        assert!(matches!(
            crate::input::resolve_prefix_binding(&config.keybinds.keybinds, &o),
            Some(crate::input::KeybindMatch::Action(
                crate::input::KeybindAction::OpenNotificationTarget
            ))
        ));
    }

    #[test]
    fn next_attention_target_walks_queue_oldest_first() {
        let mut state = attention_state();
        notify_agent(&mut state, "pane_2", "ws_2", "tab_2");
        notify_agent(&mut state, "pane_3", "ws_3", "tab_3");
        assert_eq!(queued_pane_ids(&state), ["pane_2", "pane_3"]);

        let first = record_next_attention(&mut state);
        assert_eq!(focused_pane_id(&first), Some("pane_2"));
        assert_eq!(queued_pane_ids(&state), ["pane_3"]);

        let second = record_next_attention(&mut state);
        assert_eq!(focused_pane_id(&second), Some("pane_3"));
        assert!(state.attention_targets.is_empty());

        let third = record_next_attention(&mut state);
        assert!(third.actions.is_empty());
        assert!(!third.repaint);
    }

    #[test]
    fn focusing_a_queued_agent_without_the_jump_binding_removes_it() {
        let mut state = attention_state();
        notify_agent(&mut state, "pane_2", "ws_2", "tab_2");
        notify_agent(&mut state, "pane_3", "ws_3", "tab_3");

        let mut focused = attention_snapshot();
        focused.revision = 2;
        focused.focused_workspace_id = Some("ws_2".into());
        focused.focused_tab_id = Some("tab_2".into());
        focused.focused_pane_id = Some("pane_2".into());
        focused.panes[0].focused = false;
        focused.panes[1].focused = true;
        state.set_snapshot(Box::new(focused));

        assert_eq!(queued_pane_ids(&state), ["pane_3"]);

        let jump = record_next_attention(&mut state);
        assert_eq!(focused_pane_id(&jump), Some("pane_3"));
        assert!(state.attention_targets.is_empty());
    }

    #[test]
    fn last_agent_notification_survives_toast_expiry() {
        let mut state = attention_state();
        let now = std::time::Instant::now();
        state.receive_notification(
            &ClientEndpointId::Local,
            ping("pane_2", "ws_2", "tab_2"),
            now,
        );
        assert!(state.visible_notification.is_some());
        state.tick_notifications(now + std::time::Duration::from_secs(10));
        assert!(state.visible_notification.is_none());
        assert_eq!(queued_pane_ids(&state), ["pane_2"]);

        let jump = record_next_attention(&mut state);
        assert_eq!(focused_pane_id(&jump), Some("pane_2"));
    }

    #[test]
    fn cycle_pane_next_ignores_the_attention_queue() {
        let mut state = attention_state();
        notify_agent(&mut state, "pane_2", "ws_2", "tab_2");

        let mut cycle = ClientShellInput::default();
        state.record_binding(
            crate::input::KeybindMatch::Action(crate::input::KeybindAction::CyclePaneNext),
            &mut cycle,
        );
        assert_eq!(focused_pane_id(&cycle), Some("pane_1"));
        assert_eq!(queued_pane_ids(&state), ["pane_2"]);
    }

    #[test]
    fn repeated_pings_keep_the_queue_position() {
        let mut state = attention_state();
        notify_agent(&mut state, "pane_2", "ws_2", "tab_2");
        notify_agent(&mut state, "pane_3", "ws_3", "tab_3");
        notify_agent(&mut state, "pane_2", "ws_2", "tab_2");
        assert_eq!(queued_pane_ids(&state), ["pane_2", "pane_3"]);
    }

    #[test]
    fn open_notification_target_without_toast_does_not_touch_the_queue() {
        let mut state = attention_state();
        let now = std::time::Instant::now();
        state.receive_notification(
            &ClientEndpointId::Local,
            ping("pane_2", "ws_2", "tab_2"),
            now,
        );
        state.tick_notifications(now + std::time::Duration::from_secs(10));
        assert!(state.visible_notification.is_none());
        assert_eq!(queued_pane_ids(&state), ["pane_2"]);

        let mut open = ClientShellInput::default();
        state.record_binding(
            crate::input::KeybindMatch::Action(crate::input::KeybindAction::OpenNotificationTarget),
            &mut open,
        );
        assert!(open.actions.is_empty());
        assert_eq!(queued_pane_ids(&state), ["pane_2"]);
    }

    #[test]
    fn same_tab_split_pane_ping_stays_in_the_jump_queue() {
        let mut state = attention_state();
        let mut snapshot = attention_snapshot();
        let mut pane = snapshot.panes[0].clone();
        pane.pane_id = "pane_split".into();
        pane.focused = false;
        snapshot.panes.push(pane);
        snapshot
            .agents
            .push(blocked_agent("pane_split", "ws_1", "tab_1"));
        snapshot.revision = 2;
        state.set_snapshot(Box::new(snapshot.clone()));

        notify_agent(&mut state, "pane_split", "ws_1", "tab_1");
        notify_agent(&mut state, "pane_1", "ws_1", "tab_1");
        assert_eq!(queued_pane_ids(&state), ["pane_split"]);

        snapshot.revision = 3;
        state.set_snapshot(Box::new(snapshot));
        assert_eq!(queued_pane_ids(&state), ["pane_split"]);

        state.handle_input_bytes(&[0x02]);
        let jump = state.handle_input_bytes(b"\t");
        assert_eq!(focused_pane_id(&jump), Some("pane_split"));
        assert!(state.attention_targets.is_empty());
    }

    #[test]
    fn prefix_tab_key_events_jump_to_the_next_queued_agent() {
        let mut state = attention_state();
        notify_agent(&mut state, "pane_2", "ws_2", "tab_2");
        notify_agent(&mut state, "pane_3", "ws_3", "tab_3");

        assert_eq!(state.mode, ClientShellMode::Terminal);
        let prefix = state.handle_input_bytes(&[0x02]);
        assert!(prefix.actions.is_empty());
        assert_eq!(state.mode, ClientShellMode::Prefix);

        let jump = state.handle_input_bytes(b"\t");
        assert_eq!(focused_pane_id(&jump), Some("pane_2"));
        assert_eq!(state.mode, ClientShellMode::Terminal);
        assert_eq!(queued_pane_ids(&state), ["pane_3"]);
    }

    #[test]
    fn finished_agent_stays_queued_after_toast_expiry() {
        let mut state = attention_state();
        let mut snapshot = attention_snapshot();
        snapshot.agents[0].agent_status = crate::api::schema::AgentStatus::Done;
        snapshot.agents[0].state_change_seq = 2;
        snapshot.revision = 2;
        state.set_snapshot(Box::new(snapshot));

        let now = std::time::Instant::now();
        state.receive_notification(
            &ClientEndpointId::Local,
            ping_finished("pane_2", "ws_2", "tab_2"),
            now,
        );
        assert_eq!(queued_pane_ids(&state), ["pane_2"]);
        state.tick_notifications(now + std::time::Duration::from_secs(10));
        assert!(state.visible_notification.is_none());
        assert_eq!(queued_pane_ids(&state), ["pane_2"]);
    }

    #[test]
    fn replacing_the_visible_toast_keeps_both_queue_entries() {
        let mut state = attention_state();
        notify_agent(&mut state, "pane_2", "ws_2", "tab_2");
        assert_eq!(
            state
                .visible_notification
                .as_ref()
                .and_then(|visible| visible.event.pane_id.as_deref()),
            Some("pane_2")
        );
        notify_agent(&mut state, "pane_3", "ws_3", "tab_3");
        assert_eq!(queued_pane_ids(&state), ["pane_2", "pane_3"]);
    }

    #[test]
    fn pane_focus_request_alone_does_not_remove_the_entry_but_the_snapshot_does() {
        let mut state = attention_state();
        notify_agent(&mut state, "pane_2", "ws_2", "tab_2");

        let mut requested = ClientShellInput::default();
        state.push_endpoint_method(
            crate::api::schema::Method::PaneFocus(crate::api::schema::PaneTarget {
                pane_id: "pane_2".into(),
            }),
            &mut requested,
        );
        assert_eq!(focused_pane_id(&requested), Some("pane_2"));
        assert_eq!(queued_pane_ids(&state), ["pane_2"]);

        let mut focused = attention_snapshot();
        focused.revision = 2;
        focused.focused_workspace_id = Some("ws_2".into());
        focused.focused_tab_id = Some("tab_2".into());
        focused.focused_pane_id = Some("pane_2".into());
        focused.panes[0].focused = false;
        focused.panes[1].focused = true;
        state.set_snapshot(Box::new(focused));
        assert!(state.attention_targets.is_empty());
    }

    #[test]
    fn unfocused_outer_terminal_keeps_the_focused_pane_queued() {
        let mut state = attention_state();
        state.outer_focused = Some(false);
        notify_agent(&mut state, "pane_2", "ws_2", "tab_2");

        let mut focused = attention_snapshot();
        focused.revision = 2;
        focused.focused_workspace_id = Some("ws_2".into());
        focused.focused_tab_id = Some("tab_2".into());
        focused.focused_pane_id = Some("pane_2".into());
        focused.panes[0].focused = false;
        focused.panes[1].focused = true;
        state.set_snapshot(Box::new(focused));
        assert_eq!(queued_pane_ids(&state), ["pane_2"]);

        let gained = state.handle_raw_events(vec![RawInputEvent::OuterFocusGained]);
        assert!(gained.actions.is_empty());
        assert!(state.attention_targets.is_empty());
    }

    #[test]
    fn background_endpoint_snapshot_does_not_acknowledge_focus() {
        let mut state = attention_state();
        let profile = SavedSshEndpoint {
            id: crate::client::endpoint::ProfileId::parse("0123456789abcdef0123456789abcdef")
                .expect("valid profile id"),
            label: "Build".into(),
            target: "dev@build.example".into(),
            session: "agents".into(),
            enabled: true,
        };
        let endpoint_id = ClientEndpointId::Ssh(profile.id.clone());
        state.set_endpoint_catalog(&[profile]);
        state.set_endpoint_status(&endpoint_id, ClientEndpointStatus::Online);

        let mut remote = attention_snapshot();
        remote.boot_id = "remote-boot".into();
        remote.focused_workspace_id = Some("ws_2".into());
        remote.focused_tab_id = Some("tab_2".into());
        remote.focused_pane_id = Some("pane_2".into());
        remote.panes[0].focused = false;
        remote.panes[1].focused = true;
        state.cache_endpoint_snapshot_inactive_for_generation(
            &endpoint_id,
            1,
            Box::new(remote.clone()),
        );

        let now = std::time::Instant::now();
        state.receive_notification(&endpoint_id, ping("pane_2", "ws_2", "tab_2"), now);
        assert_eq!(queued_pane_ids(&state), ["pane_2"]);

        remote.revision = 2;
        state.cache_endpoint_snapshot_inactive_for_generation(&endpoint_id, 1, Box::new(remote));
        assert_eq!(queued_pane_ids(&state), ["pane_2"]);

        assert!(state.activate_endpoint_projection(&endpoint_id));
        assert!(state.attention_targets.is_empty());
    }

    #[test]
    fn closed_pane_is_pruned_from_the_queue() {
        let mut state = attention_state();
        notify_agent(&mut state, "pane_2", "ws_2", "tab_2");
        notify_agent(&mut state, "pane_3", "ws_3", "tab_3");

        let mut snapshot = attention_snapshot();
        snapshot.revision = 2;
        snapshot.panes.retain(|pane| pane.pane_id != "pane_3");
        snapshot.tabs.retain(|tab| tab.tab_id != "tab_3");
        snapshot
            .workspaces
            .retain(|workspace| workspace.workspace_id != "ws_3");
        snapshot.agents.retain(|agent| agent.pane_id != "pane_3");
        state.set_snapshot(Box::new(snapshot));
        assert_eq!(queued_pane_ids(&state), ["pane_2"]);

        let jump = record_next_attention(&mut state);
        assert_eq!(focused_pane_id(&jump), Some("pane_2"));
        assert!(state.attention_targets.is_empty());
    }

    #[test]
    fn direct_focus_removes_only_that_entry() {
        let mut state = attention_state();
        let mut snapshot = attention_snapshot();
        push_attention_pane(&mut snapshot, "pane_4", "ws_4", "tab_4");
        snapshot.revision = 2;
        state.set_snapshot(Box::new(snapshot.clone()));

        notify_agent(&mut state, "pane_2", "ws_2", "tab_2");
        notify_agent(&mut state, "pane_3", "ws_3", "tab_3");
        notify_agent(&mut state, "pane_4", "ws_4", "tab_4");
        assert_eq!(queued_pane_ids(&state), ["pane_2", "pane_3", "pane_4"]);

        snapshot.revision = 3;
        snapshot.focused_workspace_id = Some("ws_3".into());
        snapshot.focused_tab_id = Some("tab_3".into());
        snapshot.focused_pane_id = Some("pane_3".into());
        snapshot.panes[0].focused = false;
        snapshot.panes[2].focused = true;
        state.set_snapshot(Box::new(snapshot));
        assert_eq!(queued_pane_ids(&state), ["pane_2", "pane_4"]);

        let first = record_next_attention(&mut state);
        assert_eq!(focused_pane_id(&first), Some("pane_2"));
        let second = record_next_attention(&mut state);
        assert_eq!(focused_pane_id(&second), Some("pane_4"));
        assert!(state.attention_targets.is_empty());
    }
}
