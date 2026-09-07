import "./Toggle.css";

export type Tone = "green" | "yellow" | "red";

type Props = {
  checked: boolean;
  /** Color of the track when checked. Reflects R/Y/G file status. */
  tone?: Tone;
  disabled?: boolean;
  onChange?: (next: boolean) => void;
  ariaLabel?: string;
};

export default function Toggle({
  checked,
  tone = "green",
  disabled,
  onChange,
  ariaLabel,
}: Props) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={ariaLabel}
      disabled={disabled}
      className={`toggle ${checked ? "is-on" : "is-off"} toggle--${tone}`}
      onClick={() => !disabled && onChange?.(!checked)}
    >
      <span className="toggle__track" />
      <span className="toggle__knob" />
    </button>
  );
}
