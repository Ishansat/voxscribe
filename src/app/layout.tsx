import type { Metadata } from "next";
import "../styles/globals.css";

export const metadata: Metadata = {
  title: "VoxScribe",
  description: "Privacy-first, local-first desktop meeting assistant",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
