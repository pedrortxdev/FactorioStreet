package main

import (
	"database/sql"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v4"
	"golang.org/x/crypto/bcrypt"
)

var jwtKey = []byte(getEnv("JWT_SECRET", "coprel_dev_secret_mude_em_prod"))

type Credentials struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

type Claims struct {
	Username string `json:"username"`
	UserID   int    `json:"user_id"`
	jwt.RegisteredClaims
}

func handleRegister(db *sql.DB) gin.HandlerFunc {
	return func(c *gin.Context) {
		var creds Credentials
		if err := c.ShouldBindJSON(&creds); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": "Username e password obrigatórios"})
			return
		}
		if len(creds.Username) < 3 || len(creds.Password) < 4 {
			c.JSON(http.StatusBadRequest, gin.H{"error": "Username min 3 chars, password min 4"})
			return
		}

		hash, err := bcrypt.GenerateFromPassword([]byte(creds.Password), bcrypt.DefaultCost)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "Erro interno"})
			return
		}

		_, err = db.Exec("INSERT INTO users (username, password_hash) VALUES ($1, $2)", creds.Username, string(hash))
		if err != nil {
			c.JSON(http.StatusConflict, gin.H{"error": "Username já existe"})
			return
		}

		c.JSON(http.StatusCreated, gin.H{"message": "Conta criada com sucesso"})
	}
}

func handleLogin(db *sql.DB) gin.HandlerFunc {
	return func(c *gin.Context) {
		var creds Credentials
		if err := c.ShouldBindJSON(&creds); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": "Payload inválido"})
			return
		}

		var id int
		var hash string
		err := db.QueryRow("SELECT id, password_hash FROM users WHERE username=$1", creds.Username).Scan(&id, &hash)
		if err != nil {
			c.JSON(http.StatusUnauthorized, gin.H{"error": "Credenciais inválidas"})
			return
		}

		if err := bcrypt.CompareHashAndPassword([]byte(hash), []byte(creds.Password)); err != nil {
			c.JSON(http.StatusUnauthorized, gin.H{"error": "Credenciais inválidas"})
			return
		}

		claims := &Claims{
			Username: creds.Username,
			UserID:   id,
			RegisteredClaims: jwt.RegisteredClaims{
				ExpiresAt: jwt.NewNumericDate(time.Now().Add(24 * time.Hour)),
				IssuedAt:  jwt.NewNumericDate(time.Now()),
			},
		}

		token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
		tokenString, err := token.SignedString(jwtKey)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "Erro gerando token"})
			return
		}

		c.JSON(http.StatusOK, gin.H{"token": tokenString, "user_id": id, "username": creds.Username})
	}
}

func authMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		tokenStr := c.GetHeader("Authorization")
		if len(tokenStr) > 7 && tokenStr[:7] == "Bearer " {
			tokenStr = tokenStr[7:]
		}
		if tokenStr == "" {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "Token ausente"})
			return
		}

		claims := &Claims{}
		token, err := jwt.ParseWithClaims(tokenStr, claims, func(t *jwt.Token) (interface{}, error) {
			return jwtKey, nil
		})
		if err != nil || !token.Valid {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "Token inválido"})
			return
		}

		c.Set("username", claims.Username)
		c.Set("user_id", claims.UserID)
		c.Next()
	}
}

func validateToken(tokenStr string) (*Claims, bool) {
	claims := &Claims{}
	token, err := jwt.ParseWithClaims(tokenStr, claims, func(t *jwt.Token) (interface{}, error) {
		return jwtKey, nil
	})
	if err != nil || !token.Valid {
		return nil, false
	}
	return claims, true
}
