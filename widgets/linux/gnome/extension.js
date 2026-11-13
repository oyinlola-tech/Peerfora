import Clutter from 'gi://Clutter';
import St from 'gi://St';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

export default class PeerforaCommunityExtension extends Extension {
  enable() {
    this.indicator = new St.Label({
      text: 'Peerfora preview',
      y_align: Clutter.ActorAlign.CENTER,
    });
    this._box = new St.BoxLayout({ style_class: 'panel-status-menu-box' });
    this._box.add_child(this.indicator);
    this._button = new St.Button({
      child: this._box,
      reactive: true,
      can_focus: true,
      track_hover: true,
      accessible_name: 'Peerfora community preview',
    });
    Main.panel.addToStatusArea(this.uuid, this._button);
  }

  disable() {
    this._button?.destroy();
    this._button = null;
    this._box = null;
    this.indicator = null;
  }
}
