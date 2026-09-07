import type { ReactNode } from "react";
import StatusDot, { type Status } from "./StatusDot";

type Props = {
  title?: string;
  status?: Status;
  children: ReactNode;
};

export default function Card({ title, status, children }: Props) {
  return (
    <section className="card glass">
      {title && (
        <div className="card__head">
          <span className="card__title">{title}</span>
          {status && <StatusDot status={status} />}
        </div>
      )}
      {children}
    </section>
  );
}
