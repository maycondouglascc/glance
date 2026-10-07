use gtk::gdk;
use gtk::glib;

const SORT_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
  <g stroke="#D0D0D0" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
    <line x1="2" y1="4" x2="9" y2="4"/>
    <line x1="2" y1="8" x2="7" y2="8"/>
    <line x1="2" y1="12" x2="5" y2="12"/>
    <line x1="13" y1="3" x2="13" y2="13"/>
    <polyline points="10.5,10.5 13,13 15.5,10.5" fill="none"/>
  </g>
</svg>"##;

const INFO_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
  <circle cx="8" cy="3.5" r="1.2" fill="#A1A1AA"/>
  <rect x="6.9" y="6.5" width="2.2" height="6.5" rx="1.1" fill="#A1A1AA"/>
</svg>"##;

pub fn sort_icon() -> gtk::Image {
    let bytes = glib::Bytes::from_static(SORT_SVG);
    let texture = gdk::Texture::from_bytes(&bytes).expect("Valid sort SVG");
    let img = gtk::Image::from_paintable(Some(&texture));
    img.set_pixel_size(16);
    img
}

pub fn info_icon() -> gtk::Image {
    let bytes = glib::Bytes::from_static(INFO_SVG);
    let texture = gdk::Texture::from_bytes(&bytes).expect("Valid info SVG");
    let img = gtk::Image::from_paintable(Some(&texture));
    img.set_pixel_size(16);
    img
}
