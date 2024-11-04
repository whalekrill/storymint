import axios from 'axios'
import authService from './auth'

const api = axios.create({
  baseURL: 'http://your-django-backend.com/api',
})

api.interceptors.request.use(
  (config) => {
    const token = authService.getCurrentToken()
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  },
)

export default api

import axios from 'axios'
import { jwtDecode } from 'jwt-decode'
import { API_URL } from './consts'

interface AuthResponse {
  access: string
  refresh: string
}

const authService = {
  login: async (username: string, password: string) => {
    const response = await axios.post<AuthResponse>(`${API_URL}/token/`, {
      username,
      password,
    })

    if (response.data.access) {
      localStorage.setItem('token', response.data.access)
      localStorage.setItem('refreshToken', response.data.refresh)
    }

    return response.data
  },

  refreshToken: async () => {
    const refreshToken = localStorage.getItem('refreshToken')
    const response = await axios.post<AuthResponse>(`${API_URL}/token/refresh/`, {
      refresh: refreshToken,
    })

    if (response.data.access) {
      localStorage.setItem('token', response.data.access)
    }

    return response.data
  },

  logout: () => {
    localStorage.removeItem('token')
    localStorage.removeItem('refreshToken')
  },

  getCurrentToken: () => localStorage.getItem('token'),
}

export default authService
