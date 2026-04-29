"""
Easter Egg Games Module

Hidden games for the curious. Not documented in main API docs.
Reinforces row-column thinking - critical for DataFrame operations.

Inspired by: Chrome dinosaur game, Python's antigravity, apt-get moo
"""

import random


def print_board(board):
    """Print a 3x3 tic-tac-toe board"""
    print("\n")
    for i in range(3):
        row = " | ".join(board[i])
        print(" " + row)
        if i < 2:
            print("---+---+---")
    print("\n")


def check_winner(board, player):
    """Check if a player has won"""
    win_states = [
        [(0,0),(0,1),(0,2)],  # Row 1
        [(1,0),(1,1),(1,2)],  # Row 2
        [(2,0),(2,1),(2,2)],  # Row 3
        [(0,0),(1,0),(2,0)],  # Col 1
        [(0,1),(1,1),(2,1)],  # Col 2
        [(0,2),(1,2),(2,2)],  # Col 3
        [(0,0),(1,1),(2,2)],  # Diagonal \
        [(0,2),(1,1),(2,0)]   # Diagonal /
    ]
    return any(all(board[r][c] == player for r, c in combo) for combo in win_states)


def is_full(board):
    """Check if board is full"""
    return all(board[r][c] != " " for r in range(3) for c in range(3))


def get_empty_cells(board):
    """Get list of empty cells"""
    return [(r, c) for r in range(3) for c in range(3) if board[r][c] == " "]


def computer_move(board):
    """AI move for tic-tac-toe"""
    # 1. Try to win
    for r, c in get_empty_cells(board):
        board[r][c] = "O"
        if check_winner(board, "O"):
            return
        board[r][c] = " "
    
    # 2. Try to block user
    for r, c in get_empty_cells(board):
        board[r][c] = "X"
        if check_winner(board, "X"):
            board[r][c] = "O"
            return
        board[r][c] = " "
    
    # 3. Otherwise pick random
    r, c = random.choice(get_empty_cells(board))
    board[r][c] = "O"


def tictactoe():
    """
    Play Tic-Tac-Toe against the computer.
    
    Reinforces row-column thinking - enter moves as "row col" (e.g., "2 3")
    Just like DataFrame indexing: df.iloc[row, col]
    """
    board = [[" " for _ in range(3)] for _ in range(3)]
    
    print("=" * 50)
    print("Welcome to Tic Tac Toe!")
    print("=" * 50)
    print("You are X. Computer is O.")
    print("Enter moves as: row col (e.g., '2 3' for row 2, column 3)")
    print("Rows and columns are numbered 1-3")
    print("Think of it like DataFrame indexing: df.iloc[row, col]")
    print("=" * 50)
    
    print_board(board)
    
    while True:
        # USER MOVE
        try:
            move = input("Your move (row col): ")
            r, c = map(int, move.split())
            r -= 1  # Convert to 0-indexed
            c -= 1
            
            if r not in range(3) or c not in range(3):
                print("Invalid position. Choose row/col between 1 and 3.")
                continue
            
            if board[r][c] != " ":
                print("That spot is already taken. Try again.")
                continue
            
            board[r][c] = "X"
            print_board(board)
            
            if check_winner(board, "X"):
                print("🎉 You win!")
                break
            
            if is_full(board):
                print("It's a draw!")
                break
            
            # COMPUTER MOVE
            print("Computer is thinking...")
            computer_move(board)
            print_board(board)
            
            if check_winner(board, "O"):
                print("💻 Computer wins!")
                break
            
            if is_full(board):
                print("It's a draw!")
                break
                
        except ValueError:
            print("Invalid input. Enter row and column like: 2 3")
        except KeyboardInterrupt:
            print("\n\nGame interrupted. Thanks for playing!")
            break


# A simple valid completed Sudoku board
BASE_BOARD = [
    [5,3,4,6,7,8,9,1,2],
    [6,7,2,1,9,5,3,4,8],
    [1,9,8,3,4,2,5,6,7],
    [8,5,9,7,6,1,4,2,3],
    [4,2,6,8,5,3,7,9,1],
    [7,1,3,9,2,4,8,5,6],
    [9,6,1,5,3,7,2,8,4],
    [2,8,7,4,1,9,6,3,5],
    [3,4,5,2,8,6,1,7,9]
]


def remove_numbers(board, holes=40):
    """Remove numbers from a completed Sudoku board to create a puzzle"""
    puzzle = [row[:] for row in board]
    removed = 0
    while removed < holes:
        r = random.randint(0, 8)
        c = random.randint(0, 8)
        if puzzle[r][c] != 0:
            puzzle[r][c] = 0
            removed += 1
    return puzzle


def print_sudoku_board(board):
    """Print a Sudoku board with nice formatting"""
    print("\nSudoku Board:")
    for i, row in enumerate(board):
        if i % 3 == 0 and i != 0:
            print("------+-------+------")
        row_str = ""
        for j, val in enumerate(row):
            if j % 3 == 0 and j != 0:
                row_str += "| "
            row_str += (str(val) if val != 0 else ".") + " "
        print(row_str)
    print()


def is_valid_sudoku(board, r, c, num):
    """Check if placing num at (r, c) is valid"""
    # Check row
    if num in board[r]:
        return False
    
    # Check column
    for i in range(9):
        if board[i][c] == num:
            return False
    
    # Check 3x3 box
    br = (r // 3) * 3
    bc = (c // 3) * 3
    for i in range(br, br + 3):
        for j in range(bc, bc + 3):
            if board[i][j] == num:
                return False
    
    return True


def is_solved(board):
    """Check if Sudoku is completely solved"""
    return all(all(cell != 0 for cell in row) for row in board)


def sudoku():
    """
    Play Sudoku!
    
    Reinforces row-column thinking - enter moves as "row col number" (e.g., "3 4 9")
    Just like DataFrame operations: df.iloc[row, col] = value
    """
    print("=" * 50)
    print("Welcome to Sudoku!")
    print("=" * 50)
    print("Enter moves as: row col number (e.g., '3 4 9')")
    print("Rows and columns are numbered 1-9")
    print("Think of it like DataFrame assignment: df.iloc[row, col] = value")
    print("Type 'abort' or 'exit' to quit.")
    print("=" * 50)
    
    solution = BASE_BOARD
    puzzle = remove_numbers(solution, holes=45)
    board = [row[:] for row in puzzle]
    
    print_sudoku_board(board)
    
    while True:
        move = input("Your move: ").strip().lower()
        
        if move in ("abort", "exit"):
            print("Game ended by user.")
            break
        
        try:
            r, c, num = map(int, move.split())
            r -= 1  # Convert to 0-indexed
            c -= 1
            
            if not (0 <= r < 9 and 0 <= c < 9):
                print("Invalid position. Row/col must be 1–9.")
                continue
            
            if not (1 <= num <= 9):
                print("Number must be between 1 and 9.")
                continue
            
            if puzzle[r][c] != 0:
                print("This cell is fixed and cannot be changed.")
                continue
            
            if not is_valid_sudoku(board, r, c, num):
                print("Invalid move. Violates Sudoku rules.")
                continue
            
            board[r][c] = num
            print_sudoku_board(board)
            
            if is_solved(board):
                print("🎉 Congratulations! You solved the Sudoku.")
                break
                
        except ValueError:
            print("Invalid input. Use: row col number (e.g., 2 5 7)")
        except KeyboardInterrupt:
            print("\n\nGame interrupted. Thanks for playing!")
            break


def play(game="tictactoe"):
    """
    Play a game! 🎮
    
    Available games:
    - 'tictactoe' or 'ttt': Play Tic-Tac-Toe
    - 'sudoku': Play Sudoku
    
    Both games reinforce row-column thinking - critical for DataFrame operations!
    
    Args:
        game: Name of the game to play (default: 'tictactoe')
        
    Example:
        >>> import additory as add
        >>> add.set(play='tictactoe')  # Starts the game
        >>> add.set(play='sudoku')     # Starts the game
    """
    game = game.lower().strip()
    
    if game in ('tictactoe', 'ttt'):
        tictactoe()
    elif game == 'sudoku':
        sudoku()
    else:
        print(f"Unknown game: {game}")
        print("Available games: 'tictactoe' (or 'ttt'), 'sudoku'")
        print("\nExample:")
        print("  add.set(play='tictactoe')")
        print("  add.set(play='sudoku')")


def list_games():
    """List available games."""
    return ['tictactoe', 'ttt', 'sudoku']
