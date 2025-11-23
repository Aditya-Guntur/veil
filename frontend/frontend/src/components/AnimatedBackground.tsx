import { useEffect, useRef } from 'react'

export const AnimatedBackground = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const mouseRef = useRef({ x: -1000, y: -1000 })

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    canvas.width = 2*window.innerWidth
    canvas.height = window.innerHeight

    // Characters to use in the cipher effect
    const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789@#$%^&*()_+-=[]{}|;:,.<>?/~`'
    
    const fontSize = 16
    const columns = Math.floor(canvas.width / fontSize)
    const interactionRadius = 150 // Cursor influence radius
    
    // Array to store the y position of each column
    const drops: number[] = []
    for (let i = 0; i < columns; i++) {
      drops[i] = Math.random() * -100
    }

    let animationFrameId: number

    const draw = () => {
      // More opaque black to make character changes more visible
      ctx.fillStyle = 'rgba(0, 0, 0, 0.1)'
      ctx.fillRect(0, 0, canvas.width, canvas.height)
      
      ctx.font = fontSize + 'px monospace'
      
      for (let i = 0; i < drops.length; i++) {
        // Random character (constantly changing)
        const text = characters[Math.floor(Math.random() * characters.length)]
        
        const x = i * fontSize
        const y = drops[i] * fontSize
        
        // Calculate distance from mouse
        const dx = x - mouseRef.current.x
        const dy = y - mouseRef.current.y
        const distance = Math.sqrt(dx * dx + dy * dy)
        
        // Base opacity - lower for more contrast
        let opacity = Math.random() * 0.2 + 0.1
        
        // Increase opacity near cursor - much more dramatic
        if (distance < interactionRadius) {
          const influence = 1 - (distance / interactionRadius)
          opacity = opacity + influence * 0.8 // More dramatic increase
        }
        
        ctx.fillStyle = `rgba(150, 150, 150, ${opacity})`
        
        // Draw the character
        ctx.fillText(text, x, y)
        
        // Reset drop to top randomly
        if (drops[i] * fontSize > canvas.height && Math.random() > 0.975) {
          drops[i] = 0
        }
        
        // Increment Y coordinate
        drops[i]++
      }

      animationFrameId = requestAnimationFrame(draw)
    }

    draw()

    const handleMouseMove = (e: MouseEvent) => {
      mouseRef.current = { x: e.clientX, y: e.clientY }
    }

    const handleMouseLeave = () => {
      mouseRef.current = { x: -1000, y: -1000 }
    }

    const handleResize = () => {
      canvas.width = window.innerWidth
      canvas.height = window.innerHeight
    }

    window.addEventListener('mousemove', handleMouseMove)
    window.addEventListener('mouseleave', handleMouseLeave)
    window.addEventListener('resize', handleResize)
    
    return () => {
      window.removeEventListener('mousemove', handleMouseMove)
      window.removeEventListener('mouseleave', handleMouseLeave)
      window.removeEventListener('resize', handleResize)
      cancelAnimationFrame(animationFrameId)
    }
  }, [])

  return (
    <canvas
      ref={canvasRef}
      className="fixed top-0 left-0 w-full h-full -z-10"
    />
  )
}