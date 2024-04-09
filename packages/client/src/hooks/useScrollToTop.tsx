// hooks/useScrollToTop.ts
import { useEffect } from 'react'
import { useLocation } from 'react-router-dom'

const useScrollToTop = () => {
  const location = useLocation()
  useEffect(() => {
    const scrollContainer = document.querySelector('.scroll-container')
    if (scrollContainer) {
      scrollContainer.scrollTo(0, 0)
    }
  }, [location.pathname])
}

export default useScrollToTop