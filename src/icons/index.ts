import { defineComponent, h, type PropType } from "vue";
import { nucleo } from "./nucleo";

// Inline Nucleo icons as Vue components (same names and `size` prop the app used
// with Lucide). The secondary color (`.nc-c2`) follows the theme accent.
function icon(name: string) {
  return defineComponent({
    name: `Icon${name}`,
    props: { size: { type: [Number, String] as PropType<number | string>, default: 18 } },
    setup(props) {
      const [viewBox, body] = nucleo[name] ?? ["0 0 24 24", ""];
      return () =>
        h("svg", {
          xmlns: "http://www.w3.org/2000/svg",
          viewBox,
          width: props.size,
          height: props.size,
          class: "nc-icon",
          "aria-hidden": "true",
          innerHTML: body,
        });
    },
  });
}

export const Activity = icon("Activity");
export const AlarmClockOff = icon("AlarmClockOff");
export const ArrowLeft = icon("ArrowLeft");
export const ArrowRight = icon("ArrowRight");
export const Ban = icon("Ban");
export const BarChart3 = icon("BarChart3");
export const Bell = icon("Bell");
export const Bolt = icon("Bolt");
export const Bookmark = icon("Bookmark");
export const Bot = icon("Bot");
export const Brain = icon("Brain");
export const Calendar = icon("Calendar");
export const Chat = icon("Chat");
export const Check = icon("Check");
export const CircleAlert = icon("CircleAlert");
export const CircleCheck = icon("CircleCheck");
export const Clapperboard = icon("Clapperboard");
export const Clock = icon("Clock");
export const Coins = icon("Coins");
export const Copy = icon("Copy");
export const Cpu = icon("Cpu");
export const Crown = icon("Crown");
export const Download = icon("Download");
export const ExternalLink = icon("ExternalLink");
export const Eye = icon("Eye");
export const EyeOff = icon("EyeOff");
export const FlaskConical = icon("FlaskConical");
export const FolderOpen = icon("FolderOpen");
export const Gamepad2 = icon("Gamepad2");
export const Gift = icon("Gift");
export const Globe = icon("Globe");
export const Hammer = icon("Hammer");
export const Heart = icon("Heart");
export const Incognito = icon("Incognito");
export const Info = icon("Info");
export const Key = icon("Key");
export const LayoutDashboard = icon("LayoutDashboard");
export const Layers = icon("Layers");
export const Loader2 = icon("Loader2");
export const Lock = icon("Lock");
export const LogOut = icon("LogOut");
export const Megaphone = icon("Megaphone");
export const MessageSquarePlus = icon("MessageSquarePlus");
export const Mic = icon("Mic");
export const MonitorPlay = icon("MonitorPlay");
export const Moon = icon("Moon");
export const Palette = icon("Palette");
export const Pencil = icon("Pencil");
export const Pin = icon("Pin");
export const Play = icon("Play");
export const Plus = icon("Plus");
export const Radio = icon("Radio");
export const RefreshCw = icon("RefreshCw");
export const Save = icon("Save");
export const Scissors = icon("Scissors");
export const Search = icon("Search");
export const Send = icon("Send");
export const Settings = icon("Settings");
export const Shield = icon("Shield");
export const ShieldAlert = icon("ShieldAlert");
export const ShieldCheck = icon("ShieldCheck");
export const Smile = icon("Smile");
export const Sparkles = icon("Sparkles");
export const Square = icon("Square");
export const Star = icon("Star");
export const Sun = icon("Sun");
export const Survey = icon("Survey");
export const Swords = icon("Swords");
export const Tag = icon("Tag");
export const TimerReset = icon("TimerReset");
export const Train = icon("Train");
export const Trash2 = icon("Trash2");
export const TriangleAlert = icon("TriangleAlert");
export const Trophy = icon("Trophy");
export const Undo2 = icon("Undo2");
export const UserAdd = icon("UserAdd");
export const UserTime = icon("UserTime");
export const Users = icon("Users");
export const Wand = icon("Wand");
export const Wifi = icon("Wifi");
export const WifiOff = icon("WifiOff");
export const X = icon("X");
