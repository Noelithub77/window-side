import { DotLottieReact, setWasmUrl } from "@lottiefiles/dotlottie-react"
import wasm from "@lottiefiles/dotlottie-web/dotlottie-player.wasm?url"

// Bundled renderer: the default loads it from a CDN, which an offline desktop app cannot rely on.
setWasmUrl(wasm)

type Props = { data: Record<string, unknown>; size: number; loop?: boolean }

export function Lottie({ data, size, loop = false }: Props) {
  return <DotLottieReact data={data} autoplay loop={loop} style={{ width: size, height: size }} />
}
