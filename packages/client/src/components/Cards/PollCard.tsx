import React, { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { PollOption } from '@/model/poll.model'
import VotesBadge from '@/components/VotesBadge'
import PollCardResult from '@/components/Cards/PollCardResult'
import { formatDate, markWinner } from '@/utils/methods'

interface PollCardProps {
  roundId: number
  pollOptions: PollOption[]
  totalVotes: number
  date: string
}

const PollCard: React.FC<PollCardProps> = ({ roundId, pollOptions, totalVotes, date }) => {
  const navigate = useNavigate()
  const [results, setResults] = useState<PollOption[]>(pollOptions)

  useEffect(() => {
    const newPollOptions = markWinner(pollOptions)
    setResults(newPollOptions)
  }, [pollOptions])

  const handleNavigation = () => {
    navigate(`/result/${roundId}`)
  }

  return (
    <div
      className='relative flex w-full cursor-pointer flex-col items-center justify-center space-y-4 rounded-3xl border-2 border-slate-600/20 bg-white/50 p-8 pt-2 shadow-lg md:max-w-[274px]'
      onClick={handleNavigation}
    >
      <div className='external-icon absolute right-4 top-4' />
      <div className='text-xs font-bold text-slate-600'>{formatDate(date)}</div>
      <div className='flex space-x-8 '>
        <PollCardResult results={results} totalVotes={totalVotes} />
      </div>

      <div className='absolute bottom-[-1rem] left-1/2 -translate-x-1/2 transform '>
        <VotesBadge totalVotes={totalVotes} />
      </div>
    </div>
  )
}

export default PollCard
