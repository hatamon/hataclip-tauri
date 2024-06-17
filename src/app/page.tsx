"use client";

import styles from "./page.module.css";
import { invoke } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";

export default function Home() {
    const [textList, setTextList] = useState<string[]>([]);

    useEffect(() => {
        async function fetchTextList() {
            try {
                const list: string[] = await invoke("get_text_list");
                setTextList(list);
            } catch (error) {
                console.error("Error fetching text list:", error);
            }
        }

        fetchTextList();
    }, []);

    return (
        <main className={styles.main}>
            <ul>
                {textList.map((text, index) => (
                    <li key={index}>{text}</li>
                ))}
            </ul>
        </main>
    );
}
