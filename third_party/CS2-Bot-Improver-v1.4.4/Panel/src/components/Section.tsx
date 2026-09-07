import type { ReactNode } from "react";
import StatusDot, { type Status } from "./StatusDot";
import "./Section.css";

type Props = {
  title: string;
  status?: Status;
  right?: ReactNode;
  children: ReactNode;
};

// A non-collapsible titled block (always shows its content).
export default function Section({ title, status, right, children }: Props) {
  return (
    <div className="section">
      <div className="section__head">
        <span className="section__title">{title}</span>
        <span className="section__right">
          {right}
          {status && <StatusDot status={status} />}
        </span>
      </div>
      <div className="section__body">{children}</div>
    </div>
  );
}
