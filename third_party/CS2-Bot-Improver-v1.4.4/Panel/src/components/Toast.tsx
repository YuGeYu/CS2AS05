import {
  createContext,
  useCallback,
  useContext,
  useRef,
  useState,
  type ReactNode,
} from "react";
import "./Toast.css";

type ToastTone = "neutral" | "green" | "red";
type ToastItem = { id: number; text: string; tone: ToastTone };

type ToastApi = { show: (text: string, tone?: ToastTone) => void };

const Ctx = createContext<ToastApi>({ show: () => {} });

export function useToast() {
  return useContext(Ctx);
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<ToastItem[]>([]);
  const seq = useRef(0);

  const show = useCallback((text: string, tone: ToastTone = "neutral") => {
    const id = ++seq.current;
    setItems((prev) => [...prev, { id, text, tone }]);
    window.setTimeout(() => {
      setItems((prev) => prev.filter((t) => t.id !== id));
    }, 1900);
  }, []);

  return (
    <Ctx.Provider value={{ show }}>
      {children}
      <div className="toast__stack">
        {items.map((t) => (
          <div key={t.id} className={`toast toast--${t.tone} glass glass-strong`}>
            {t.text}
          </div>
        ))}
      </div>
    </Ctx.Provider>
  );
}
