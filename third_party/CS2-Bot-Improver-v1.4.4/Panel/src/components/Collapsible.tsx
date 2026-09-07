import { useState, type ReactNode } from "react";
import { ChevronDown } from "./icons";
import StatusDot, { type Status } from "./StatusDot";
import "./Collapsible.css";

type Props = {
  title: string;
  status?: Status;
  defaultOpen?: boolean;
  right?: ReactNode;
  children: ReactNode;
};

export default function Collapsible({
  title,
  status,
  defaultOpen = false,
  right,
  children,
}: Props) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className={`collapsible ${open ? "is-open" : ""}`}>
      <button className="collapsible__head" onClick={() => setOpen((o) => !o)}>
        <span className="collapsible__title">{title}</span>
        <span className="collapsible__right">
          {right}
          {status && <StatusDot status={status} />}
          <ChevronDown size={18} className="collapsible__chev" />
        </span>
      </button>
      <div className="collapsible__body" hidden={!open}>
        {children}
      </div>
    </div>
  );
}
