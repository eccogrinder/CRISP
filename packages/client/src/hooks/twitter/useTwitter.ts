import { handleGenericError } from '@/utils/handle-generic-error'
import { Twitter, SocialAuth } from '@/model/twitter.model'
import useLocalStorage from '@/hooks/generic/useLocalStorage'
import { AUTH_MSG } from '@/pages/Register/Register'
import { useVoteManagementContext } from '@/context/voteManagement'
import { useApi } from '@/hooks/generic/useFetchApi'

const TWITTER_API = import.meta.env.VITE_TWITTER_SERVERLESS_API

if (!TWITTER_API) handleGenericError('useTwitter', { name: 'TWITTER_API', message: 'Missing env VITE_TWITTER_SERVERLESS_API' })

export const useTwitter = () => {
  const url = `${TWITTER_API}/twitter-data`
  const { fetchData, isLoading } = useApi()
  const [_socialAuth, setSocialAuth] = useLocalStorage<SocialAuth | null>('socialAuth', null)
  const { setUser } = useVoteManagementContext()

  const extractUsernameFromUrl = (url: string): string | null => {
    const regex = /https:\/\/[^\/]+\/([^\/]+)\/status\/\d+/
    const match = url.match(regex)
    return match ? match[1] : null
  }

  const handleTwitterPostVerification = async (postUrl: string) => {
    const username = extractUsernameFromUrl(postUrl)
    const result = await verifyPost(postUrl)
    if (result) {
      const descriptionLowerCase = result.description.toLowerCase()
      const authMsgLowerCase = AUTH_MSG.toLowerCase()
      if (descriptionLowerCase.includes(authMsgLowerCase)) {
        const user = {
          validationDate: new Date(),
          avatar: result.open_graph.images[0].url ?? '',
          username: username ?? '',
        }
        setUser(user)
        setSocialAuth(user)
      }
    }
  }

  //Api
  const verifyPost = (postUrl: string) => fetchData<Twitter, { url: string }>(url, 'post', { url: postUrl })

  return {
    isLoading,
    handleTwitterPostVerification,
  }
}
